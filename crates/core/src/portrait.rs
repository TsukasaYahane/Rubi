//! 竖屏推流 straight from the recommendation pool. The pool is sorted into portrait and landscape
//! as it fills: lists that give a picture size say so at once, the rest are looked up one by one
//! at a gentle pace (two at a time), and every answer is kept so it is never asked again. Picking
//! takes the portrait ones in ranked order; when they run short a new batch is fetched and
//! sorted, and only then are videos related to the ones played added. A refusal by risk control
//! pauses the looking up for a while: it is waited out, never worked around.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::future::join_all;
use rubi_api::{ApiError, FeedApi};
use rubi_model::VideoCard;
use rubi_store::KvRepo;

use crate::error::{CoreError, Result};
use crate::ports::{PortraitPool, RuleSource, ShapeLookup, keep_cards};
use crate::videos::portrait_of;

const KEY_SHAPES: &str = "shapes";
/// Shapes remembered; the oldest go first.
const SHAPES_KEPT: usize = 20_000;
/// How far down the ranked pool picking and sorting look.
const LOOK_AHEAD: usize = 240;
/// Lookups at once, and per round of picking.
const AT_ONCE: usize = 2;
const PER_ROUND: usize = 24;
/// Rounds of picking before related videos are asked for.
const ROUNDS: usize = 3;
/// How long lookups wait after risk control said no.
const PAUSE: Duration = Duration::from_secs(10 * 60);
/// Videos played whose related lists are looked through.
const CHAIN_LOOKED: usize = 3;

/// What a round of sorting did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sorted {
    pub looked: usize,
    pub portrait: usize,
    /// Risk control said no: looking up waits a while.
    pub limited: bool,
}

/// Portrait videos to play next.
#[derive(Debug, Clone, Default)]
pub struct Picked {
    pub cards: Vec<VideoCard>,
    /// Risk control holds the sorting back; these are what was ready.
    pub limited: bool,
}

/// Shapes learned, oldest first.
#[derive(Default)]
struct Shapes {
    map: HashMap<String, bool>,
    order: VecDeque<String>,
}

impl Shapes {
    fn from_list(list: Vec<(String, bool)>) -> Self {
        let mut shapes = Self::default();
        for (bvid, portrait) in list {
            shapes.put(bvid, portrait);
        }
        shapes
    }

    fn put(&mut self, bvid: String, portrait: bool) {
        if self.map.insert(bvid.clone(), portrait).is_none() {
            self.order.push_back(bvid);
        }
        while self.order.len() > SHAPES_KEPT {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
    }

    fn list(&self) -> Vec<(String, bool)> {
        self.order.iter().filter_map(|b| self.map.get(b).map(|p| (b.clone(), *p))).collect()
    }
}

#[derive(Default)]
struct State {
    shapes: Option<Shapes>,
    /// Played in this run of the feed.
    served: HashSet<String>,
    /// The latest of them, for related videos.
    chain: VecDeque<String>,
    paused_until: Option<Instant>,
}

pub struct PortraitFeed {
    pool: Arc<dyn PortraitPool>,
    shapes: Arc<dyn ShapeLookup>,
    related: Arc<dyn FeedApi>,
    rules: Arc<dyn RuleSource>,
    kv: Arc<dyn KvRepo + Send + Sync>,
    state: Mutex<State>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl PortraitFeed {
    pub(crate) fn new(
        pool: Arc<dyn PortraitPool>,
        shapes: Arc<dyn ShapeLookup>,
        related: Arc<dyn FeedApi>,
        rules: Arc<dyn RuleSource>,
        kv: Arc<dyn KvRepo + Send + Sync>,
    ) -> Self {
        Self { pool, shapes, related, rules, kv, state: Mutex::default() }
    }

    /// Starts a new run of the feed: what was played before may come again.
    pub fn start(&self) {
        let mut state = lock(&self.state);
        state.served.clear();
        state.chain.clear();
    }

    fn with_shapes<R>(&self, f: impl FnOnce(&mut Shapes) -> R) -> R {
        let mut state = lock(&self.state);
        let shapes = state.shapes.get_or_insert_with(|| {
            let list: Vec<(String, bool)> = self
                .kv
                .get_raw(KEY_SHAPES)
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            Shapes::from_list(list)
        });
        f(shapes)
    }

    /// Whether `card` is portrait, as far as is known without asking.
    fn shape_of(&self, card: &VideoCard) -> Option<bool> {
        portrait_of(&card.raw)
            .or_else(|| self.with_shapes(|s| s.map.get(&card.bvid).copied()))
            .or_else(|| self.shapes.known_portrait(&card.bvid))
    }

    fn paused(&self) -> bool {
        lock(&self.state).paused_until.is_some_and(|until| Instant::now() < until)
    }

    /// Sorts up to `budget` pooled videos whose shape is not known yet, best ranked first.
    pub async fn sort(&self, budget: usize) -> Result<Sorted> {
        let mut sorted = Sorted::default();
        if self.paused() {
            sorted.limited = true;
            return Ok(sorted);
        }
        let pooled = self.pool.ranked(LOOK_AHEAD, &HashSet::new())?;
        let unknown: Vec<String> =
            pooled.iter().filter(|c| self.shape_of(c).is_none()).map(|c| c.bvid.clone()).take(budget).collect();
        let mut learned = Vec::new();
        'chunks: for chunk in unknown.chunks(AT_ONCE) {
            let answers = join_all(chunk.iter().map(|bvid| self.shapes.look_up_portrait(bvid))).await;
            for (bvid, answer) in chunk.iter().zip(answers) {
                match answer {
                    Ok(portrait) => {
                        sorted.looked += 1;
                        sorted.portrait += usize::from(portrait);
                        learned.push((bvid.clone(), portrait));
                    }
                    Err(CoreError::Api(ApiError::RiskControl { .. })) => {
                        lock(&self.state).paused_until = Some(Instant::now() + PAUSE);
                        sorted.limited = true;
                        break 'chunks;
                    }
                    // One video that cannot be looked up (gone, private) is left for later.
                    Err(_) => {}
                }
            }
        }
        if !learned.is_empty() {
            let list = self.with_shapes(|s| {
                for (bvid, portrait) in learned {
                    s.put(bvid, portrait);
                }
                s.list()
            });
            self.kv.put_raw(KEY_SHAPES, &serde_json::to_string(&list)?)?;
        }
        Ok(sorted)
    }

    /// The portrait videos known in the pool, best ranked first, not played yet.
    fn take_known(&self, want: usize) -> Result<Vec<VideoCard>> {
        let served = lock(&self.state).served.clone();
        let pooled = self.pool.ranked(LOOK_AHEAD, &served)?;
        Ok(pooled.into_iter().filter(|c| self.shape_of(c) == Some(true)).take(want).collect())
    }

    /// Portrait videos related to the latest played, as their lists give the size.
    async fn related_portraits(&self, want: usize) -> Vec<VideoCard> {
        let (chain, served) = {
            let state = lock(&self.state);
            (state.chain.iter().rev().take(CHAIN_LOOKED).cloned().collect::<Vec<_>>(), state.served.clone())
        };
        let mut found: Vec<VideoCard> = Vec::new();
        for bvid in chain {
            let Ok(cards) = self.related.related(&bvid).await else { continue };
            let Ok((kept, _)) = keep_cards(self.rules.as_ref(), cards) else { continue };
            for card in kept {
                let fresh = !served.contains(&card.bvid) && !found.iter().any(|f| f.bvid == card.bvid);
                if fresh && portrait_of(&card.raw) == Some(true) {
                    found.push(card);
                }
            }
            if found.len() >= want {
                break;
            }
        }
        found.truncate(want);
        found
    }

    /// Up to `want` portrait videos to play next: from the pool first, sorting more of it and
    /// fetching a new batch when it runs short, then videos related to the ones played.
    pub async fn next(&self, want: usize) -> Result<Picked> {
        let mut picked = Picked::default();
        let mut fetched = false;
        for _ in 0..ROUNDS {
            let more = self.take_known(want - picked.cards.len())?;
            self.mark_served(&more);
            picked.cards.extend(more);
            if picked.cards.len() >= want {
                return Ok(picked);
            }
            let sorted = self.sort(PER_ROUND).await?;
            if sorted.limited {
                picked.limited = true;
                break;
            }
            if sorted.looked == 0 {
                // Everything in reach is sorted: a new batch, once.
                if fetched {
                    break;
                }
                self.pool.fetch_more().await?;
                fetched = true;
            }
        }
        let more = self.take_known(want - picked.cards.len())?;
        self.mark_served(&more);
        picked.cards.extend(more);
        if picked.cards.len() < want {
            let related = self.related_portraits(want - picked.cards.len()).await;
            self.mark_served(&related);
            picked.cards.extend(related);
        }
        Ok(picked)
    }

    fn mark_served(&self, cards: &[VideoCard]) {
        let mut state = lock(&self.state);
        for card in cards {
            state.served.insert(card.bvid.clone());
            state.chain.push_back(card.bvid.clone());
        }
        while state.chain.len() > CHAIN_LOOKED * 4 {
            state.chain.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use rubi_store::SqliteStore;
    use serde_json::json;

    use super::*;
    use crate::settings::SettingsService;

    fn card(bvid: &str, shape: Option<bool>) -> VideoCard {
        let raw = match shape {
            Some(true) => json!({"dimension": {"width": 1080, "height": 1920, "rotate": 0}}),
            Some(false) => json!({"dimension": {"width": 1920, "height": 1080, "rotate": 0}}),
            None => json!({}),
        };
        VideoCard { bvid: bvid.into(), raw, ..VideoCard::default() }
    }

    /// The pool as ranked cards; a fetch adds the next batch.
    struct Pool {
        cards: Mutex<Vec<VideoCard>>,
        batch: Mutex<Vec<VideoCard>>,
        fetches: AtomicUsize,
    }

    #[async_trait]
    impl PortraitPool for Pool {
        fn ranked(&self, n: usize, skip: &HashSet<String>) -> Result<Vec<VideoCard>> {
            Ok(lock(&self.cards).iter().filter(|c| !skip.contains(&c.bvid)).take(n).cloned().collect())
        }
        async fn fetch_more(&self) -> Result<()> {
            self.fetches.fetch_add(1, Ordering::Relaxed);
            let batch = std::mem::take(&mut *lock(&self.batch));
            lock(&self.cards).extend(batch);
            Ok(())
        }
    }

    /// Answers lookups from a table; refuses once `refuse_after` have been answered.
    struct Lookup {
        portrait: HashSet<String>,
        asked: AtomicUsize,
        refuse_after: usize,
    }

    #[async_trait]
    impl ShapeLookup for Lookup {
        fn known_portrait(&self, _: &str) -> Option<bool> {
            None
        }
        async fn look_up_portrait(&self, bvid: &str) -> Result<bool> {
            if self.asked.fetch_add(1, Ordering::Relaxed) >= self.refuse_after {
                return Err(ApiError::RiskControl { code: -412, message: "no".into() }.into());
            }
            Ok(self.portrait.contains(bvid))
        }
    }

    struct Related(Vec<VideoCard>);

    #[async_trait]
    impl FeedApi for Related {
        async fn home_rcmd(&self, _: u32, _: u32) -> rubi_api::Result<Vec<VideoCard>> {
            Ok(Vec::new())
        }
        async fn popular(&self, _: u32, _: u32) -> rubi_api::Result<Vec<VideoCard>> {
            Ok(Vec::new())
        }
        async fn ranking(&self, _: u32) -> rubi_api::Result<Vec<VideoCard>> {
            Ok(Vec::new())
        }
        async fn related(&self, _: &str) -> rubi_api::Result<Vec<VideoCard>> {
            Ok(self.0.clone())
        }
        async fn region_feed(&self, _: u32, _: u32, _: u32) -> rubi_api::Result<Vec<VideoCard>> {
            Ok(Vec::new())
        }
        async fn followed_videos(&self, _: Option<&str>) -> rubi_api::Result<(Vec<VideoCard>, Option<String>)> {
            Ok((Vec::new(), None))
        }
    }

    fn feed(
        pool: Vec<VideoCard>,
        batch: Vec<VideoCard>,
        portrait: &[&str],
        refuse_after: usize,
        related: Vec<VideoCard>,
    ) -> (PortraitFeed, Arc<Pool>, Arc<Lookup>, Arc<SqliteStore>) {
        let pool = Arc::new(Pool { cards: Mutex::new(pool), batch: Mutex::new(batch), fetches: AtomicUsize::new(0) });
        let lookup = Arc::new(Lookup {
            portrait: portrait.iter().map(|s| s.to_string()).collect(),
            asked: AtomicUsize::new(0),
            refuse_after,
        });
        let store = Arc::new(SqliteStore::in_memory().unwrap());
        let rules = Arc::new(SettingsService::open(Arc::new(SqliteStore::in_memory().unwrap())).unwrap());
        let feed = PortraitFeed::new(pool.clone(), lookup.clone(), Arc::new(Related(related)), rules, store.clone());
        (feed, pool, lookup, store)
    }

    fn ids(cards: &[VideoCard]) -> Vec<&str> {
        cards.iter().map(|c| c.bvid.as_str()).collect()
    }

    #[tokio::test]
    async fn sized_cards_are_picked_without_asking() {
        let pool = vec![card("a", Some(false)), card("b", Some(true)), card("c", Some(true))];
        let (feed, _, lookup, _) = feed(pool, Vec::new(), &[], usize::MAX, Vec::new());
        let picked = feed.next(2).await.unwrap();
        assert_eq!(ids(&picked.cards), ["b", "c"]);
        assert_eq!(lookup.asked.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn unknown_cards_are_looked_up_once_and_kept() {
        let pool = vec![card("a", None), card("b", None), card("c", None)];
        let (feed, _, lookup, store) = feed(pool, Vec::new(), &["b"], usize::MAX, Vec::new());
        let picked = feed.next(1).await.unwrap();
        assert_eq!(ids(&picked.cards), ["b"]);
        assert_eq!(lookup.asked.load(Ordering::Relaxed), 3);
        assert!(store.get_raw(KEY_SHAPES).unwrap().unwrap().contains("\"b\",true"));
        feed.start();
        assert_eq!(ids(&feed.next(1).await.unwrap().cards), ["b"]);
        assert_eq!(lookup.asked.load(Ordering::Relaxed), 3);
    }

    #[tokio::test]
    async fn a_new_batch_comes_when_the_pool_runs_short() {
        let (feed, pool, _, _) =
            feed(vec![card("a", Some(false))], vec![card("n", Some(true))], &[], usize::MAX, Vec::new());
        assert_eq!(ids(&feed.next(1).await.unwrap().cards), ["n"]);
        assert_eq!(pool.fetches.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn related_videos_fill_in_last() {
        let (feed, _, _, _) = feed(
            vec![card("p", Some(true))],
            Vec::new(),
            &[],
            usize::MAX,
            vec![card("r1", Some(false)), card("r2", Some(true)), card("p", Some(true))],
        );
        let picked = feed.next(3).await.unwrap();
        assert_eq!(ids(&picked.cards), ["p", "r2"]);
    }

    #[tokio::test]
    async fn risk_control_pauses_the_lookups() {
        let pool = vec![card("a", None), card("b", None), card("c", None), card("d", Some(true))];
        let (feed, _, lookup, _) = feed(pool, Vec::new(), &["a"], 1, Vec::new());
        let picked = feed.next(3).await.unwrap();
        assert!(picked.limited);
        assert_eq!(ids(&picked.cards), ["d", "a"]);
        let asked = lookup.asked.load(Ordering::Relaxed);
        assert!(feed.sort(10).await.unwrap().limited);
        assert_eq!(lookup.asked.load(Ordering::Relaxed), asked);
    }

    #[test]
    fn shapes_forget_the_oldest() {
        let mut shapes = Shapes::default();
        for n in 0..SHAPES_KEPT + 2 {
            shapes.put(format!("v{n}"), n % 2 == 0);
        }
        assert_eq!(shapes.map.len(), SHAPES_KEPT);
        assert!(!shapes.map.contains_key("v0"));
        assert_eq!(shapes.list().first().map(|(b, _)| b.as_str()), Some("v2"));
    }
}
