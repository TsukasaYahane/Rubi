//! The home feed: pulls candidates from every source, filters them, stores the pool
//! and ranks pages out of it with the local engine.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures_util::future::join_all;
use rubi_filter::{Category, Rule, RuleSet, WatchedRule};
use rubi_model::{DislikeReason, FeedbackEvent, ItemRef, Signal, SourceKind, VideoCard, now_secs};
use rubi_recommend::{
    Candidate, FeedEngine, InterestProfile, Ranked, ScoreBreakdown, ScoringContext, Source, default_factors,
};
use rubi_store::{
    CandidateRepo, EventRepo, ExposureRepo, FilterLogEntry, FilterLogRepo, HousekeepingRepo, KvRepo, WatchedRepo,
};

use crate::error::Result;
use crate::ports::{FeedConfig, FeedbackSink, HomeFeed, ProfileBackup, WatchedSet};

const KEY_PROFILE: &str = "profile";
const KEY_FOLLOWED: &str = "followed";
/// Videos marked 不感兴趣: never offered again.
const KEY_DISLIKED: &str = "disliked";
/// How many 不感兴趣 videos are remembered (oldest forgotten first).
const MAX_DISLIKED: usize = 5000;
/// Signals that make a video a seed for 相关推荐 (completion counts when mostly watched).
const SEED_KINDS: &[&str] = &["like", "coin", "favorite", "rewatch", "completion"];

/// Everything the store has to offer the feed.
pub trait AppStore:
    CandidateRepo
    + ExposureRepo
    + EventRepo
    + WatchedRepo
    + FilterLogRepo
    + HousekeepingRepo
    + KvRepo
    + Send
    + Sync
    + 'static
{
}
impl<
    T: CandidateRepo
        + ExposureRepo
        + EventRepo
        + WatchedRepo
        + FilterLogRepo
        + HousekeepingRepo
        + KvRepo
        + Send
        + Sync
        + 'static,
> AppStore for T
{
}

/// A pull to refresh fetches a source again only after this many seconds, however often the
/// user pulls.
const MANUAL_REFRESH_FLOOR_S: u64 = 5;

#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    pub fetched: Vec<(SourceKind, usize)>,
    pub skipped: Vec<SourceKind>,
    pub kept: usize,
    pub blocked: usize,
    pub errors: Vec<String>,
}

/// What the profile says the user likes, derived before each refresh.
#[derive(Debug, Clone, Default)]
pub struct Seeds {
    /// Videos recently liked, coined, favourited, rewatched or mostly finished; newest first.
    pub liked: Vec<String>,
    /// The main regions liked most.
    pub zones: Vec<u32>,
}

/// A source whose inputs come from the profile: told the seeds before each refresh.
pub trait Seedable: Send + Sync {
    fn seed(&self, seeds: &Seeds);
}

/// 不感兴趣 videos: in the order marked (to forget the oldest), and as a set to look up.
#[derive(Default)]
struct Disliked {
    order: VecDeque<String>,
    set: HashSet<String>,
}

impl Disliked {
    fn from_list(list: Vec<String>) -> Self {
        let set = list.iter().cloned().collect();
        Self { order: list.into(), set }
    }

    /// False when already listed.
    fn add(&mut self, bvid: &str) -> bool {
        if !self.set.insert(bvid.to_owned()) {
            return false;
        }
        self.order.push_back(bvid.to_owned());
        while self.order.len() > MAX_DISLIKED {
            if let Some(old) = self.order.pop_front() {
                self.set.remove(&old);
            }
        }
        true
    }
}

pub struct FeedService<S: AppStore> {
    store: Arc<S>,
    config: Arc<dyn FeedConfig>,
    sources: Vec<Arc<dyn Source>>,
    seeded: Vec<Arc<dyn Seedable>>,
    /// Lock order: `profile` / `disliked` / `shown` before the store's own lock, never after.
    profile: Mutex<InterestProfile>,
    last_fetch: Mutex<HashMap<SourceKind, Instant>>,
    /// One refresh at a time: a second waits and then finds the sources cooling down.
    refreshing: tokio::sync::Mutex<()>,
    /// Cards handed out by `next_page` since the last `new_session`: the visible list, which
    /// later pages must not repeat.
    shown: Mutex<HashSet<String>>,
    disliked: Mutex<Disliked>,
}

fn category_name(c: Category) -> String {
    serde_json::to_value(c).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default()
}

/// Ad cards often have no title; fall back to the card type so the log stays readable.
fn display_title(card: &VideoCard) -> String {
    if !card.title.is_empty() {
        return card.title.clone();
    }
    let goto = card.raw["goto"].as_str().or_else(|| card.raw["card_goto"].as_str()).unwrap_or("unknown");
    format!("[{goto}]")
}

fn source_name(k: SourceKind) -> String {
    serde_json::to_value(k).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default()
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl<S: AppStore> FeedService<S> {
    pub fn new(
        store: Arc<S>,
        config: Arc<dyn FeedConfig>,
        sources: Vec<Arc<dyn Source>>,
        seeded: Vec<Arc<dyn Seedable>>,
    ) -> Result<Self> {
        let profile = store.get_json(KEY_PROFILE)?.unwrap_or_default();
        let disliked = Disliked::from_list(store.get_json(KEY_DISLIKED)?.unwrap_or_default());
        Ok(Self {
            store,
            config,
            sources,
            seeded,
            profile: Mutex::new(profile),
            last_fetch: Mutex::new(HashMap::new()),
            refreshing: tokio::sync::Mutex::new(()),
            shown: Mutex::new(HashSet::new()),
            disliked: Mutex::new(disliked),
        })
    }

    pub fn profile(&self) -> InterestProfile {
        lock(&self.profile).clone()
    }

    pub fn set_profile(&self, profile: InterestProfile) -> Result<()> {
        let mut p = lock(&self.profile);
        self.store.put_json(KEY_PROFILE, &profile)?;
        *p = profile;
        Ok(())
    }

    pub fn set_followed(&self, mids: &[u64]) -> Result<()> {
        Ok(self.store.put_json(KEY_FOLLOWED, &mids)?)
    }

    fn followed(&self) -> Result<HashSet<u64>> {
        Ok(self.store.get_json::<Vec<u64>>(KEY_FOLLOWED)?.unwrap_or_default().into_iter().collect())
    }

    /// The watched check for these cards, read in one go; `None` when the rules leave it off.
    fn watched_rule<'a>(
        &self,
        rules: &RuleSet,
        cards: impl Iterator<Item = &'a VideoCard>,
    ) -> Result<Option<WatchedRule>> {
        if !rules.toggles.watched {
            return Ok(None);
        }
        let ids: Vec<&str> = cards.map(|c| c.bvid.as_str()).collect();
        let watched = WatchedSet(self.store.watched_among(&ids)?);
        Ok(rules.watched_rule(Arc::new(watched)))
    }

    /// Fetches from every source that is not cooling down, all at once, and adds what survives
    /// the filters to the candidate pool.
    pub async fn refresh(&self, want_per_source: usize) -> Result<RefreshReport> {
        let cooldown = Duration::from_secs(self.config.feed_settings().min_source_interval_s);
        self.fetch(want_per_source, cooldown).await
    }

    /// [`refresh`](Self::refresh) asked for by the user (pull to refresh): every source is
    /// fetched again unless it was fetched in the last few seconds.
    pub async fn refresh_now(&self, want_per_source: usize) -> Result<RefreshReport> {
        self.fetch(want_per_source, Duration::from_secs(MANUAL_REFRESH_FLOOR_S)).await
    }

    async fn fetch(&self, want_per_source: usize, cooldown: Duration) -> Result<RefreshReport> {
        let _single = self.refreshing.lock().await;
        self.update_seeds()?;
        let settings = self.config.feed_settings();
        let mut report = RefreshReport::default();

        let due: Vec<&Arc<dyn Source>> = {
            let last = lock(&self.last_fetch);
            self.sources
                .iter()
                .filter(|src| {
                    let due = last.get(&src.kind()).is_none_or(|t| t.elapsed() >= cooldown);
                    if !due {
                        report.skipped.push(src.kind());
                    }
                    due
                })
                .collect()
        };
        let results =
            join_all(due.iter().map(|src| async move { (src.kind(), src.fetch(want_per_source).await) })).await;

        let mut fetched: Vec<(SourceKind, VideoCard)> = Vec::new();
        {
            let mut last = lock(&self.last_fetch);
            for (kind, result) in results {
                match result {
                    Ok(cards) => {
                        last.insert(kind, Instant::now());
                        report.fetched.push((kind, cards.len()));
                        fetched.extend(cards.into_iter().map(|c| (kind, c)));
                    }
                    Err(e) => report.errors.push(e.to_string()),
                }
            }
        }

        let rules = self.config.rules();
        let filter = self.config.card_filter();
        let watched = self.watched_rule(&rules, fetched.iter().map(|(_, c)| c))?;
        let extra = watched.as_ref().map(|r| r as &dyn Rule<VideoCard>);
        let now = now_secs();
        let mut accepted = Vec::new();
        let mut log = Vec::new();
        {
            let disliked = lock(&self.disliked);
            for (kind, mut card) in fetched {
                if disliked.set.contains(&card.bvid) {
                    report.blocked += 1;
                    continue;
                }
                card.add_source(kind);
                let decision = filter.judge_with(&card, extra);
                match decision.blocked_by {
                    Some(hit) => {
                        report.blocked += 1;
                        log.push(FilterLogEntry {
                            at: now,
                            rule_id: hit.rule_id,
                            category: category_name(hit.category),
                            title: display_title(&card),
                            source: source_name(kind),
                            reason: hit.reason,
                        });
                    }
                    None => accepted.push((card, decision.multiplier)),
                }
            }
        }

        report.kept = accepted.len();
        self.store.put_candidates(&accepted, settings.candidate_ttl_s, now)?;
        self.store.append_filter_log(&log)?;
        self.store.purge_expired(now)?;
        self.store.prune(now)?;
        Ok(report)
    }

    /// Starts a new visible list (pull to refresh): earlier pages may be ranked again.
    pub fn new_session(&self) {
        lock(&self.shown).clear();
    }

    /// Ranks the pool and returns the next `n` cards not yet shown in this session, recording
    /// them as shown. Empty when the pool has nothing new; refresh to fetch more. Two calls
    /// at once never hand out the same card: the shown set is held throughout.
    pub fn next_page(&self, n: usize) -> Result<Vec<Ranked>> {
        let mut shown = lock(&self.shown);
        let ranked = self.rank_pool(n, &shown)?;
        let ids: Vec<String> = ranked.iter().map(|r| r.card.bvid.clone()).collect();
        self.store.record_impressions(&ids, now_secs())?;
        shown.extend(ids);
        Ok(ranked)
    }

    /// The score breakdown of a pooled card, for "为什么推荐".
    pub fn explain(&self, bvid: &str) -> Result<Option<ScoreBreakdown>> {
        let Some(candidate) = self.candidates(&HashSet::new())?.into_iter().find(|c| c.card.bvid == bvid) else {
            return Ok(None);
        };
        let (exposures, followed, profile) = (self.store.exposures_since_click()?, self.followed()?, self.profile());
        let ctx = ScoringContext { now: now_secs(), profile: &profile, followed: &followed, exposures: &exposures };
        Ok(Some(self.engine().score(&candidate, &ctx)))
    }

    fn engine(&self) -> FeedEngine {
        let settings = self.config.feed_settings();
        FeedEngine::new(default_factors(), settings.weights, settings.source_weights)
    }

    /// The pool as it may be shown now: not excluded, not disliked, and passing the rules
    /// (judged again: rules may have changed and the user may have watched some since).
    fn candidates(&self, exclude: &HashSet<String>) -> Result<Vec<Candidate>> {
        let pool: Vec<(VideoCard, f32)> = {
            let disliked = lock(&self.disliked);
            self.store
                .fresh_candidates(now_secs())?
                .into_iter()
                .filter(|(card, _)| !exclude.contains(&card.bvid) && !disliked.set.contains(&card.bvid))
                .collect()
        };
        let rules = self.config.rules();
        let filter = self.config.card_filter();
        let watched = self.watched_rule(&rules, pool.iter().map(|(c, _)| c))?;
        let extra = watched.as_ref().map(|r| r as &dyn Rule<VideoCard>);
        Ok(pool
            .into_iter()
            .filter_map(|(card, penalty)| {
                let d = filter.judge_with(&card, extra);
                (!d.is_blocked()).then(|| Candidate { penalty: penalty.min(d.multiplier), card })
            })
            .collect())
    }

    fn rank_pool(&self, n: usize, exclude: &HashSet<String>) -> Result<Vec<Ranked>> {
        let candidates = self.candidates(exclude)?;
        let exposures = self.store.exposures_since_click()?;
        let followed = self.followed()?;
        let profile = self.profile();
        let ctx = ScoringContext { now: now_secs(), profile: &profile, followed: &followed, exposures: &exposures };
        Ok(self.engine().rank(candidates, &ctx, n))
    }

    /// Records a feedback signal and updates the profile.
    pub fn record(&self, ev: FeedbackEvent) -> Result<()> {
        self.store.append_event(&ev)?;
        self.apply_marks(&ev)?;
        // Held while writing, so a later event's profile is never overwritten by an earlier one.
        let mut p = lock(&self.profile);
        p.apply(&ev);
        Ok(self.store.put_json(KEY_PROFILE, &*p)?)
    }

    /// What an event leaves besides the profile: watched, 不感兴趣, followed. Safe to repeat.
    fn apply_marks(&self, ev: &FeedbackEvent) -> Result<()> {
        let Some(item) = &ev.item else { return Ok(()) };
        match &ev.signal {
            Signal::Click => {
                self.store.record_click(&item.bvid)?;
                self.store.mark_watched(&item.bvid, ev.at)?;
            }
            Signal::Dislike { reason } => self.dislike(item, reason, ev.at)?,
            Signal::Follow => {
                let mid = item.up_mid;
                self.store.update_json(KEY_FOLLOWED, |f: &mut Vec<u64>| {
                    if let Err(at) = f.binary_search(&mid) {
                        f.insert(at, mid);
                    }
                })?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Keeps a 不感兴趣 video out of the feed for good and lists it under 被过滤内容.
    fn dislike(&self, item: &ItemRef, reason: &DislikeReason, at: i64) -> Result<()> {
        {
            let mut d = lock(&self.disliked);
            if !d.add(&item.bvid) {
                return Ok(());
            }
            self.store.put_json(KEY_DISLIKED, &d.order)?;
        }
        let why = match reason {
            DislikeReason::Up => "不想看这个 UP".to_owned(),
            DislikeReason::Zone => "不想看这个分区".to_owned(),
            DislikeReason::Tag(t) if !t.is_empty() => format!("不想看「{t}」"),
            DislikeReason::Tag(_) => "不想看这类标签".to_owned(),
            DislikeReason::Title => "标题不感兴趣".to_owned(),
        };
        Ok(self.store.append_filter_log(&[FilterLogEntry {
            at,
            rule_id: "user.dislike".into(),
            category: "dislike".into(),
            title: if item.title.is_empty() { item.bvid.clone() } else { item.title.clone() },
            source: "home".into(),
            reason: why,
        }])?)
    }

    /// Every recorded feedback event, oldest first (for export).
    pub fn history(&self) -> Result<Vec<FeedbackEvent>> {
        Ok(self.store.events_since(0)?)
    }

    pub fn filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>> {
        Ok(self.store.recent_filter_log(limit)?)
    }

    fn update_seeds(&self) -> Result<()> {
        if self.seeded.is_empty() {
            return Ok(());
        }
        let since = now_secs() - 7 * 86_400;
        let mut liked: Vec<String> = self
            .store
            .events_of_kinds_since(since, SEED_KINDS)?
            .into_iter()
            .rev()
            .filter(|e| !matches!(e.signal, Signal::Completion { ratio } if ratio < 0.6))
            .filter_map(|e| e.item.map(|i| i.bvid))
            .collect();
        liked.dedup();
        let p = self.profile();
        let now = now_secs();
        let mut z: Vec<(u32, f64)> = p.likes.zones.iter().map(|(k, d)| (*k, d.get(now, p.half_life_days))).collect();
        z.sort_by(|a, b| b.1.total_cmp(&a.1));
        let seeds = Seeds { liked, zones: crate::zones::main_regions(z.into_iter().map(|(k, _)| k), 3) };
        for s in &self.seeded {
            s.seed(&seeds);
        }
        Ok(())
    }
}

impl<S: AppStore> FeedbackSink for FeedService<S> {
    fn record(&self, ev: FeedbackEvent) -> Result<()> {
        FeedService::record(self, ev)
    }

    fn set_followed(&self, mids: &[u64]) -> Result<()> {
        let mut sorted = mids.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        FeedService::set_followed(self, &sorted)
    }
}

#[async_trait]
impl<S: AppStore> HomeFeed for FeedService<S> {
    async fn refresh(&self, want_per_source: usize) -> Result<RefreshReport> {
        FeedService::refresh(self, want_per_source).await
    }

    async fn refresh_now(&self, want_per_source: usize) -> Result<RefreshReport> {
        FeedService::refresh_now(self, want_per_source).await
    }

    fn new_session(&self) {
        FeedService::new_session(self)
    }

    fn next_page(&self, n: usize) -> Result<Vec<Ranked>> {
        FeedService::next_page(self, n)
    }

    fn explain(&self, bvid: &str) -> Result<Option<ScoreBreakdown>> {
        FeedService::explain(self, bvid)
    }

    fn filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>> {
        FeedService::filter_log(self, limit)
    }
}

impl<S: AppStore> ProfileBackup for FeedService<S> {
    fn profile(&self) -> InterestProfile {
        FeedService::profile(self)
    }

    fn history(&self) -> Result<Vec<FeedbackEvent>> {
        FeedService::history(self)
    }

    fn restore(&self, profile: InterestProfile, history: &[FeedbackEvent]) -> Result<()> {
        if !history.is_empty() {
            self.store.replace_events(history)?;
            for ev in history {
                self.apply_marks(ev)?;
            }
        }
        self.set_profile(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::SettingsPort;
    use crate::settings::SettingsService;
    use rubi_model::{ItemRef, Owner, Stats, VideoCard};
    use rubi_recommend::SourceError;
    use rubi_store::SqliteStore;
    use serde_json::json;

    struct Fixed(SourceKind, Vec<VideoCard>);

    #[async_trait]
    impl Source for Fixed {
        fn kind(&self) -> SourceKind {
            self.0
        }
        async fn fetch(&self, _: usize) -> std::result::Result<Vec<VideoCard>, SourceError> {
            Ok(self.1.clone())
        }
    }

    fn video(bvid: &str, mid: u64) -> VideoCard {
        VideoCard {
            bvid: bvid.into(),
            title: format!("video {bvid}"),
            owner: Owner { mid, name: format!("up{mid}") },
            pubdate: now_secs() - 3600,
            stats: Stats { view: 1000, like: 40, ..Default::default() },
            raw: json!({"goto": "av"}),
            ..Default::default()
        }
    }

    fn service() -> FeedService<SqliteStore> {
        let ad = VideoCard { title: "广告".into(), raw: json!({"goto": "ad"}), ..Default::default() };
        let sources: Vec<Arc<dyn Source>> = vec![
            Arc::new(Fixed(SourceKind::BiliRecommend, vec![video("a", 1), ad])),
            Arc::new(Fixed(SourceKind::Popular, vec![video("a", 1), video("b", 2), video("c", 3)])),
        ];
        let store = Arc::new(SqliteStore::in_memory().unwrap());
        let config = Arc::new(SettingsService::open(store.clone()).unwrap());
        FeedService::new(store, config, sources, Vec::new()).unwrap()
    }

    #[tokio::test]
    async fn refresh_filters_logs_and_ranks() {
        let svc = service();
        let r = svc.refresh(10).await.unwrap();
        assert_eq!(r.blocked, 1);
        assert_eq!(r.kept, 4);
        assert_eq!(svc.filter_log(10).unwrap()[0].category, "ad");

        let page = svc.next_page(10).unwrap();
        assert_eq!(page.len(), 3);
        let a = page.iter().find(|r| r.card.bvid == "a").unwrap();
        assert_eq!(a.card.sources.len(), 2, "merged from both sources");

        let again = svc.refresh(10).await.unwrap();
        assert_eq!(again.skipped.len(), 2, "sources are cooling down");
    }

    #[tokio::test]
    async fn pages_do_not_repeat_within_a_session() {
        let svc = service();
        svc.refresh(10).await.unwrap();
        let first: Vec<String> = svc.next_page(2).unwrap().into_iter().map(|r| r.card.bvid).collect();
        let second: Vec<String> = svc.next_page(2).unwrap().into_iter().map(|r| r.card.bvid).collect();
        assert_eq!(first.len() + second.len(), 3);
        assert!(second.iter().all(|b| !first.contains(b)));
        assert!(svc.next_page(2).unwrap().is_empty(), "pool used up");
        svc.new_session();
        assert_eq!(svc.next_page(10).unwrap().len(), 3);
    }

    #[tokio::test]
    async fn concurrent_refreshes_ask_each_source_once() {
        let svc = service();
        let (a, b) = tokio::join!(svc.refresh(10), svc.refresh(10));
        let fetched = a.unwrap().fetched.len() + b.unwrap().fetched.len();
        assert_eq!(fetched, 2, "the second refresh finds both sources cooling down");
    }

    #[tokio::test]
    async fn watched_and_disliked_items_drop() {
        let svc = service();
        svc.refresh(10).await.unwrap();
        let item = |b: &str, m: u64| Some(ItemRef { bvid: b.into(), up_mid: m, ..Default::default() });
        svc.record(FeedbackEvent { signal: Signal::Click, item: item("a", 1), at: now_secs() }).unwrap();
        svc.record(FeedbackEvent {
            signal: Signal::Dislike { reason: rubi_model::DislikeReason::Up },
            item: item("b", 2),
            at: now_secs(),
        })
        .unwrap();
        let page = svc.next_page(10).unwrap();
        let ids: Vec<&str> = page.iter().map(|r| r.card.bvid.as_str()).collect();
        assert_eq!(ids, vec!["c"], "a is watched, b is disliked so it never comes back");
        svc.new_session();
        svc.refresh(10).await.unwrap();
        assert!(svc.next_page(10).unwrap().iter().all(|r| r.card.bvid != "b"), "not even after a refresh");
        assert_eq!(svc.filter_log(5).unwrap()[0].rule_id, "user.dislike");
        assert!(svc.explain("c").unwrap().is_some());
        assert!(svc.explain("b").unwrap().is_none());
    }

    #[test]
    fn follows_merge_and_restore_does_not_count_twice() {
        let svc = service();
        let follow = |mid| FeedbackEvent {
            signal: Signal::Follow,
            item: Some(ItemRef { up_mid: mid, ..Default::default() }),
            at: 1,
        };
        svc.record(follow(5)).unwrap();
        svc.record(follow(3)).unwrap();
        svc.record(follow(5)).unwrap();
        assert_eq!(svc.followed().unwrap(), HashSet::from([3, 5]));

        let like = FeedbackEvent {
            signal: Signal::Like,
            item: Some(ItemRef { bvid: "x".into(), ..Default::default() }),
            at: 1,
        };
        svc.record(like.clone()).unwrap();
        let (profile, history) = (svc.profile(), svc.history().unwrap());
        svc.restore(profile.clone(), &history).unwrap();
        assert_eq!(svc.profile(), profile, "the profile is taken as it is");
        assert_eq!(svc.history().unwrap(), history, "events are not duplicated");
    }

    #[test]
    fn settings_reach_the_feed() {
        let store = Arc::new(SqliteStore::in_memory().unwrap());
        let config = Arc::new(SettingsService::open(store.clone()).unwrap());
        let mut s = config.settings();
        s.feed.min_source_interval_s = 0;
        config.set_settings(s).unwrap();
        let svc = FeedService::new(store, config, Vec::new(), Vec::new()).unwrap();
        assert_eq!(svc.config.feed_settings().min_source_interval_s, 0);
    }
}
