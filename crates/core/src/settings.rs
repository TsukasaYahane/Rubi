//! The settings object: owned here, read by the services through narrow ports, and changed
//! only through [`SettingsService`], which compiles the rule table once per change.
//!
//! On disk and over FFI the settings stay one flat JSON object; in code they are split by who
//! reads them: the rule table, the feed's own knobs, and what the player and lists show.

use std::sync::{Arc, Mutex, RwLock};

use rubi_filter::danmaku::DanmakuOptions;
use rubi_filter::{Pipeline, RuleSet};
use rubi_model::{VideoCard, now_secs};
use rubi_recommend::{SourceWeights, Weights};
use rubi_store::KvRepo;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{CoreError, Result};
use crate::ports::{FeedConfig, RuleSource, SettingsPort};

const KEY_SETTINGS: &str = "settings";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub rules: RuleSet,
    #[serde(flatten)]
    pub feed: FeedSettings,
    #[serde(flatten)]
    pub playback: PlaybackSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self { rules: RuleSet::builtin(), feed: FeedSettings::default(), playback: PlaybackSettings::default() }
    }
}

/// What the home feed is tuned by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FeedSettings {
    pub weights: Weights,
    pub source_weights: SourceWeights,
    /// How long a fetched candidate stays in the pool.
    pub candidate_ttl_s: i64,
    /// A source is not asked again within this interval (risk-control courtesy).
    pub min_source_interval_s: u64,
}

impl Default for FeedSettings {
    fn default() -> Self {
        Self {
            weights: Weights::default(),
            source_weights: SourceWeights::default(),
            candidate_ttl_s: 6 * 3600,
            min_source_interval_s: 30,
        }
    }
}

/// What the app shows and plays; only the app reads these.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackSettings {
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

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            danmaku_display: DanmakuDisplay::default(),
            background_play: false,
            preview: PreviewSettings::default(),
            long_press_speed: 1.5,
            preferred_quality: 80,
            auto_pip: false,
        }
    }
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

/// The settings with what is built from them.
struct Compiled {
    settings: Settings,
    rules: Arc<RuleSet>,
    card_filter: Arc<Pipeline<VideoCard>>,
    danmaku: Arc<DanmakuOptions>,
}

impl Compiled {
    /// Fails on an invalid regex, so a bad table never replaces a good one.
    fn new(settings: Settings) -> Result<Self> {
        let card_filter = Arc::new(settings.rules.card_pipeline(None, now_secs)?);
        let danmaku = Arc::new(settings.rules.danmaku_options()?);
        let rules = Arc::new(settings.rules.clone());
        Ok(Self { settings, rules, card_filter, danmaku })
    }
}

/// Owns the settings: one copy in memory, the same on disk.
pub(crate) struct SettingsService<S> {
    store: Arc<S>,
    current: RwLock<Arc<Compiled>>,
    /// Held across read-change-write so two changes never overwrite each other.
    writer: Mutex<()>,
}

impl<S: KvRepo + Send + Sync> SettingsService<S> {
    pub(crate) fn open(store: Arc<S>) -> Result<Self> {
        let mut settings: Settings = store.get_json(KEY_SETTINGS)?.unwrap_or_default();
        settings.rules.merge_builtin();
        // A table stored by an older build that no longer compiles falls back to the built-in one.
        let compiled = match Compiled::new(settings.clone()) {
            Ok(c) => c,
            Err(_) => Compiled::new(Settings { rules: RuleSet::builtin(), ..settings })?,
        };
        Ok(Self { store, current: RwLock::new(Arc::new(compiled)), writer: Mutex::new(()) })
    }

    fn current(&self) -> Arc<Compiled> {
        self.current.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Validates, stores and publishes `next`. The caller holds `writer`.
    fn publish(&self, next: Settings) -> Result<()> {
        let compiled = Compiled::new(next)?;
        self.store.put_json(KEY_SETTINGS, &compiled.settings)?;
        *self.current.write().unwrap_or_else(|e| e.into_inner()) = Arc::new(compiled);
        Ok(())
    }
}

/// RFC 7386 JSON merge patch.
fn merge_patch(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(t), Value::Object(p)) => {
            for (k, v) in p {
                if v.is_null() {
                    t.remove(&k);
                } else {
                    merge_patch(t.entry(k).or_insert(Value::Null), v);
                }
            }
        }
        (t, p) => *t = p,
    }
}

impl<S: KvRepo + Send + Sync> SettingsPort for SettingsService<S> {
    fn settings(&self) -> Settings {
        self.current().settings.clone()
    }

    fn set_settings(&self, settings: Settings) -> Result<()> {
        let _w = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        self.publish(settings)
    }

    fn patch_json(&self, patch: &str) -> Result<()> {
        let patch: Value = serde_json::from_str(patch)?;
        if !patch.is_object() {
            return Err(CoreError::Invalid("settings patch must be a JSON object".into()));
        }
        let _w = self.writer.lock().unwrap_or_else(|e| e.into_inner());
        let mut doc = serde_json::to_value(&self.current().settings)?;
        merge_patch(&mut doc, patch);
        self.publish(serde_json::from_value(doc)?)
    }
}

impl<S: KvRepo + Send + Sync> RuleSource for SettingsService<S> {
    fn rules(&self) -> Arc<RuleSet> {
        self.current().rules.clone()
    }

    fn card_filter(&self) -> Arc<Pipeline<VideoCard>> {
        self.current().card_filter.clone()
    }

    fn danmaku_options(&self) -> Arc<DanmakuOptions> {
        self.current().danmaku.clone()
    }
}

impl<S: KvRepo + Send + Sync> FeedConfig for SettingsService<S> {
    fn feed_settings(&self) -> FeedSettings {
        let mut feed = self.current().settings.feed.clone();
        // 0.15 was the old default for 重复曝光, far too small to move a card the user passed
        // over. Settings saved back then carry it, so a stored default is lifted to the new one;
        // a weight the user picked on purpose is anything else and stays.
        if feed.weights.0.get(&rubi_recommend::FactorId::RepeatExposure).is_some_and(|w| *w == 0.15) {
            feed.weights.0.insert(rubi_recommend::FactorId::RepeatExposure, 0.50);
        }
        feed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubi_filter::KeywordSpec;
    use rubi_store::SqliteStore;

    fn service() -> SettingsService<SqliteStore> {
        SettingsService::open(Arc::new(SqliteStore::in_memory().unwrap())).unwrap()
    }

    #[test]
    fn json_stays_flat() {
        let v = serde_json::to_value(Settings::default()).unwrap();
        for key in ["rules", "weights", "candidate_ttl_s", "danmaku_display", "preferred_quality", "auto_pip"] {
            assert!(v.get(key).is_some(), "{key} at the top level");
        }
        let back: Settings = serde_json::from_value(v).unwrap();
        assert_eq!(back, Settings::default());
        let partial: Settings = serde_json::from_str(r#"{"auto_pip": true}"#).unwrap();
        assert!(partial.playback.auto_pip);
        assert_eq!(partial.feed, FeedSettings::default());
    }

    #[test]
    fn patches_keep_other_keys() {
        let s = service();
        s.patch_json(r#"{"danmaku_display": {"opacity": 0.5}}"#).unwrap();
        s.patch_json(r#"{"preview": {"volume": 0.3}}"#).unwrap();
        let now = s.settings();
        assert_eq!(now.playback.danmaku_display.opacity, 0.5);
        assert!(now.playback.danmaku_display.enabled, "untouched keys stay");
        assert_eq!(now.playback.preview.volume, 0.3);
    }

    #[test]
    fn invalid_rules_change_nothing() {
        let s = service();
        let mut bad = s.settings();
        bad.rules.keywords.push(KeywordSpec { pattern: "(".into(), regex: true, fields: vec![] });
        assert!(s.set_settings(bad).is_err());
        assert!(s.settings().rules.keywords.is_empty());
        assert!(s.patch_json("[1]").is_err());
    }
}
