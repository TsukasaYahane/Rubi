//! The home feed: pulls candidates from every source, filters them, stores the pool
//! and ranks pages out of it with the local engine.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use rubi_filter::{Category, RuleSet, WatchedLookup};
use rubi_model::{DislikeReason, FeedbackEvent, ItemRef, Signal, SourceKind, VideoCard, now_secs};
use rubi_recommend::{
    Candidate, FeedEngine, InterestProfile, Ranked, ScoreBreakdown, ScoringContext, Source, SourceWeights, Weights,
    default_factors,
};
use rubi_store::{CandidateRepo, EventRepo, ExposureRepo, FilterLogEntry, FilterLogRepo, KvRepo, WatchedRepo};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::sources::{PreferredZoneSource, RelatedSource};

const KEY_SETTINGS: &str = "settings";
const KEY_PROFILE: &str = "profile";
const KEY_FOLLOWED: &str = "followed";
/// Videos marked 不感兴趣: never offered again.
const KEY_DISLIKED: &str = "disliked";
/// How many 不感兴趣 videos are remembered (oldest forgotten first).
const MAX_DISLIKED: usize = 5000;

/// Everything the store has to offer the feed.
pub trait AppStore:
    CandidateRepo + ExposureRepo + EventRepo + WatchedRepo + FilterLogRepo + KvRepo + Send + Sync + 'static
{
}
impl<T: CandidateRepo + ExposureRepo + EventRepo + WatchedRepo + FilterLogRepo + KvRepo + Send + Sync + 'static>
    AppStore for T
{
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub rules: RuleSet,
    pub weights: Weights,
    pub source_weights: SourceWeights,
    /// How long a fetched candidate stays in the pool.
    pub candidate_ttl_s: i64,
    /// A source is not asked again within this interval (risk-control courtesy).
    pub min_source_interval_s: u64,
    pub danmaku_display: DanmakuDisplay,
    /// Keep playing when the app goes to the background. Off: pause instead.
    pub background_play: bool,
    pub preview: PreviewSettings,
    /// Playback speed while the fullscreen player is held down.
    pub long_press_speed: f32,
    /// Quality (qn) asked for when a video opens; the server caps it to what the account may play.
    pub preferred_quality: u32,
    /// Leaving the app from the player goes into picture-in-picture instead of pausing.
    pub auto_pip: bool,
}

/// 大图预览: the card in the middle of the single-column feed plays a short preview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PreviewSettings {
    pub enabled: bool,
    /// Only on unmetered networks (Wi-Fi), so previews never spend mobile data.
    pub wifi_only: bool,
    /// 0.0 (muted) – 1.0.
    pub volume: f32,
    pub danmaku: bool,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self { enabled: true, wifi_only: true, volume: 0.0, danmaku: false }
    }
}

/// How danmaku are drawn (the filters themselves live in the rule table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DanmakuDisplay {
    pub enabled: bool,
    /// Share of the video height danmaku may use: 0.25, 0.5, 0.75 or 1.0.
    pub area: f32,
    /// Multiplies the base 14 sp size; 0.5–2.0.
    pub font_scale: f32,
    /// 0.2–1.0.
    pub opacity: f32,
    /// Scroll speed multiplier; 0.5–2.0 (base: 8 s across the screen).
    pub speed: f32,
    pub show_scroll: bool,
    pub show_top: bool,
    pub show_bottom: bool,
    /// When false, only white danmaku are shown.
    pub show_colored: bool,
}

impl Default for DanmakuDisplay {
    fn default() -> Self {
        Self {
            enabled: true,
            area: 0.75,
            font_scale: 1.0,
            opacity: 0.9,
            speed: 1.0,
            show_scroll: true,
            show_top: true,
            show_bottom: true,
            show_colored: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rules: RuleSet::builtin(),
            weights: Weights::default(),
            source_weights: SourceWeights::default(),
            candidate_ttl_s: 6 * 3600,
            min_source_interval_s: 30,
            danmaku_display: DanmakuDisplay::default(),
            background_play: false,
            preview: PreviewSettings::default(),
            long_press_speed: 1.5,
            preferred_quality: 80,
            auto_pip: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RefreshReport {
    pub fetched: Vec<(SourceKind, usize)>,
    pub skipped: Vec<SourceKind>,
    pub kept: usize,
    pub blocked: usize,
    pub errors: Vec<String>,
}

/// Sources whose inputs are derived from the profile before each refresh.
#[derive(Default)]
pub struct Seeded {
    pub related: Option<Arc<RelatedSource>>,
    pub zones: Option<Arc<PreferredZoneSource>>,
}

struct StoreWatched<S>(Arc<S>);

impl<S: AppStore> WatchedLookup for StoreWatched<S> {
    fn is_watched(&self, bvid: &str) -> bool {
        self.0.is_watched(bvid).unwrap_or(false)
    }
}

pub struct FeedService<S: AppStore> {
    store: Arc<S>,
    sources: Vec<Arc<dyn Source>>,
    seeded: Seeded,
    settings: RwLock<Settings>,
    profile: Mutex<InterestProfile>,
    last_fetch: Mutex<HashMap<SourceKind, Instant>>,
    /// Cards handed out by `next_page` since the last `new_session`: the visible list, which
    /// later pages must not repeat.
    shown: Mutex<HashSet<String>>,
    /// 不感兴趣 videos, oldest first; kept out of the pool for good.
    disliked: Mutex<Vec<String>>,
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

impl<S: AppStore> FeedService<S> {
    pub fn new(store: Arc<S>, sources: Vec<Arc<dyn Source>>, seeded: Seeded) -> Result<Self> {
        let mut settings: Settings = store.get_json(KEY_SETTINGS)?.unwrap_or_default();
        settings.rules.merge_builtin();
        let profile = store.get_json(KEY_PROFILE)?.unwrap_or_default();
        let disliked = store.get_json(KEY_DISLIKED)?.unwrap_or_default();
        Ok(Self {
            store,
            sources,
            seeded,
            settings: RwLock::new(settings),
            profile: Mutex::new(profile),
            last_fetch: Mutex::new(HashMap::new()),
            shown: Mutex::new(HashSet::new()),
            disliked: Mutex::new(disliked),
        })
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set_settings(&self, settings: Settings) -> Result<()> {
        settings.rules.card_pipeline(None, now_secs)?; // reject invalid regexes before saving
        self.store.put_json(KEY_SETTINGS, &settings)?;
        *self.settings.write().unwrap_or_else(|e| e.into_inner()) = settings;
        Ok(())
    }

    pub fn profile(&self) -> InterestProfile {
        self.profile.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set_profile(&self, profile: InterestProfile) -> Result<()> {
        self.store.put_json(KEY_PROFILE, &profile)?;
        *self.profile.lock().unwrap_or_else(|e| e.into_inner()) = profile;
        Ok(())
    }

    pub fn set_followed(&self, mids: &[u64]) -> Result<()> {
        Ok(self.store.put_json(KEY_FOLLOWED, &mids)?)
    }

    fn followed(&self) -> Result<HashSet<u64>> {
        Ok(self.store.get_json::<Vec<u64>>(KEY_FOLLOWED)?.unwrap_or_default().into_iter().collect())
    }

    fn watched(&self) -> Arc<dyn WatchedLookup> {
        Arc::new(StoreWatched(self.store.clone()))
    }

    /// Fetches from every source that is not cooling down and adds what survives the
    /// filters to the candidate pool.
    pub async fn refresh(&self, want_per_source: usize) -> Result<RefreshReport> {
        self.update_seeds()?;
        let settings = self.settings();
        let pipeline = settings.rules.card_pipeline(Some(self.watched()), now_secs)?;
        let cooldown = Duration::from_secs(settings.min_source_interval_s);
        let mut report = RefreshReport::default();
        let mut accepted = Vec::new();
        let mut log = Vec::new();

        for src in &self.sources {
            let kind = src.kind();
            let due = self
                .last_fetch
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&kind)
                .is_none_or(|t| t.elapsed() >= cooldown);
            if !due {
                report.skipped.push(kind);
                continue;
            }
            let cards = match src.fetch(want_per_source).await {
                Ok(c) => c,
                Err(e) => {
                    report.errors.push(e.to_string());
                    continue;
                }
            };
            self.last_fetch.lock().unwrap_or_else(|e| e.into_inner()).insert(kind, Instant::now());
            report.fetched.push((kind, cards.len()));
            let now = now_secs();
            let disliked = self.disliked_set();
            for mut card in cards {
                if disliked.contains(&card.bvid) {
                    report.blocked += 1;
                    continue;
                }
                card.add_source(kind);
                let decision = pipeline.judge(&card);
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

        let now = now_secs();
        report.kept = accepted.len();
        self.store.put_candidates(&accepted, settings.candidate_ttl_s, now)?;
        self.store.append_filter_log(&log)?;
        self.store.purge_expired(now)?;
        Ok(report)
    }

    /// Starts a new visible list (pull to refresh): earlier pages may be ranked again.
    pub fn new_session(&self) {
        self.shown.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    /// Ranks the pool and returns the next `n` cards not yet shown in this session, recording
    /// them as shown. Empty when the pool has nothing new; refresh to fetch more.
    pub fn next_page(&self, n: usize) -> Result<Vec<Ranked>> {
        let exclude = self.shown.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let ranked = self.rank_pool(n, &exclude)?;
        let ids: Vec<String> = ranked.iter().map(|r| r.card.bvid.clone()).collect();
        self.store.record_impressions(&ids, now_secs())?;
        self.shown.lock().unwrap_or_else(|e| e.into_inner()).extend(ids);
        Ok(ranked)
    }

    /// The score breakdown of a pooled card, for "为什么推荐".
    pub fn explain(&self, bvid: &str) -> Result<Option<ScoreBreakdown>> {
        Ok(self.rank_pool(usize::MAX, &HashSet::new())?.into_iter().find(|r| r.card.bvid == bvid).map(|r| r.score))
    }

    fn rank_pool(&self, n: usize, exclude: &HashSet<String>) -> Result<Vec<Ranked>> {
        let settings = self.settings();
        // Re-judge: rules may have changed and the user may have watched some since.
        let pipeline = settings.rules.card_pipeline(Some(self.watched()), now_secs)?;
        let now = now_secs();
        let disliked = self.disliked_set();
        let candidates: Vec<Candidate> = self
            .store
            .fresh_candidates(now)?
            .into_iter()
            .filter(|(card, _)| !exclude.contains(&card.bvid) && !disliked.contains(&card.bvid))
            .filter_map(|(card, penalty)| {
                let d = pipeline.judge(&card);
                (!d.is_blocked()).then(|| Candidate { penalty: penalty.min(d.multiplier), card })
            })
            .collect();
        let exposures = self.store.exposures_since_click()?;
        let followed = self.followed()?;
        let profile = self.profile();
        let ctx = ScoringContext { now, profile: &profile, followed: &followed, exposures: &exposures };
        let engine = FeedEngine::new(default_factors(), settings.weights, settings.source_weights);
        Ok(engine.rank(candidates, &ctx, n))
    }

    /// Records a feedback signal and updates the profile.
    pub fn record(&self, ev: FeedbackEvent) -> Result<()> {
        self.store.append_event(&ev)?;
        if let Some(item) = &ev.item {
            match ev.signal {
                Signal::Click => {
                    self.store.record_click(&item.bvid)?;
                    self.store.mark_watched(&item.bvid, ev.at)?;
                }
                Signal::Dislike { ref reason } => self.dislike(item, reason, ev.at)?,
                Signal::Follow => {
                    let mut f: Vec<u64> = self.followed()?.into_iter().collect();
                    f.push(item.up_mid);
                    f.sort_unstable();
                    f.dedup();
                    self.set_followed(&f)?;
                }
                _ => {}
            }
        }
        let profile = {
            let mut p = self.profile.lock().unwrap_or_else(|e| e.into_inner());
            p.apply(&ev);
            p.clone()
        };
        Ok(self.store.put_json(KEY_PROFILE, &profile)?)
    }

    /// Keeps a 不感兴趣 video out of the feed for good and lists it under 被过滤内容.
    fn dislike(&self, item: &ItemRef, reason: &DislikeReason, at: i64) -> Result<()> {
        let list = {
            let mut d = self.disliked.lock().unwrap_or_else(|e| e.into_inner());
            if d.contains(&item.bvid) {
                return Ok(());
            }
            d.push(item.bvid.clone());
            let over = d.len().saturating_sub(MAX_DISLIKED);
            d.drain(..over);
            d.clone()
        };
        self.store.put_json(KEY_DISLIKED, &list)?;
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

    fn disliked_set(&self) -> HashSet<String> {
        self.disliked.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect()
    }

    /// Every recorded feedback event, oldest first (for export).
    pub fn history(&self) -> Result<Vec<FeedbackEvent>> {
        Ok(self.store.events_since(0)?)
    }

    pub fn filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>> {
        Ok(self.store.recent_filter_log(limit)?)
    }

    fn update_seeds(&self) -> Result<()> {
        if let Some(related) = &self.seeded.related {
            let since = now_secs() - 7 * 86_400;
            let mut seeds: Vec<String> = self
                .store
                .events_since(since)?
                .into_iter()
                .rev()
                .filter(|e| {
                    matches!(e.signal, Signal::Like | Signal::Coin | Signal::Favorite | Signal::Rewatch)
                        || matches!(e.signal, Signal::Completion { ratio } if ratio >= 0.6)
                })
                .filter_map(|e| e.item.map(|i| i.bvid))
                .collect();
            seeds.dedup();
            related.set_seeds(seeds);
        }
        if let Some(zones) = &self.seeded.zones {
            let p = self.profile();
            let now = now_secs();
            let mut z: Vec<(u32, f64)> =
                p.likes.zones.iter().map(|(k, d)| (*k, d.get(now, p.half_life_days))).collect();
            z.sort_by(|a, b| b.1.total_cmp(&a.1));
            zones.set_zones(crate::zones::main_regions(z.into_iter().map(|(k, _)| k), 3));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
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
        FeedService::new(Arc::new(SqliteStore::in_memory().unwrap()), sources, Seeded::default()).unwrap()
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
    }
}
