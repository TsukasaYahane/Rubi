//! What services need from each other, as narrow traits. A service holds only the ports it
//! uses and never the service behind them, so each can change (or be faked in a test) alone:
//!
//! - [`RuleSource`], [`FeedConfig`]: the rule table and its compiled filters ([`crate::settings::SettingsService`]).
//! - [`FeedbackSink`]: where user actions go to shape the profile ([`crate::feed::FeedService`]).
//! - [`HomeFeed`], [`ProfileBackup`]: the feed as the home page and the backup see it.
//! - [`SettingsPort`], [`PlaylistBackup`]: settings and play lists for the backup.
//! - [`VideoLookup`], [`VideoStreams`]: a video's card and streams ([`crate::videos::Videos`]).
//! - [`LoginState`]: whether someone is signed in ([`crate::account::Account`]).
//! - [`PortraitPool`], [`ShapeLookup`]: the pool and video shapes as the 竖屏推流 uses them
//!   ([`crate::feed::FeedService`], [`crate::videos::Videos`]).

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use rubi_filter::danmaku::DanmakuOptions;
use rubi_filter::{Pipeline, RuleSet};
use rubi_model::{FeedbackEvent, VideoCard};
use rubi_recommend::{InterestProfile, Ranked, ScoreBreakdown};
use rubi_store::FilterLogEntry;

use crate::error::Result;
use crate::feed::RefreshReport;
use crate::settings::{FeedSettings, Settings};
use crate::videos::Playback;

/// The current rule table and the filters compiled from it. Compiling happens once per change
/// of the table, never per call.
pub trait RuleSource: Send + Sync {
    fn rules(&self) -> Arc<RuleSet>;
    /// The filter for list cards (the watched check is the feed's own).
    fn card_filter(&self) -> Arc<Pipeline<VideoCard>>;
    fn danmaku_options(&self) -> Arc<DanmakuOptions>;
}

/// What the feed is tuned by, with the rule table it filters with.
pub trait FeedConfig: RuleSource {
    fn feed_settings(&self) -> FeedSettings;
}

/// Where user actions go to shape the recommendation profile.
pub trait FeedbackSink: Send + Sync {
    fn record(&self, ev: FeedbackEvent) -> Result<()>;
    fn set_followed(&self, mids: &[u64]) -> Result<()>;
}

/// The home feed as its page uses it.
#[async_trait]
pub trait HomeFeed: Send + Sync {
    async fn refresh(&self, want_per_source: usize) -> Result<RefreshReport>;
    /// A refresh the user asked for: sources are fetched again despite their cooldown.
    async fn refresh_now(&self, want_per_source: usize) -> Result<RefreshReport>;
    fn new_session(&self);
    fn next_page(&self, n: usize) -> Result<Vec<Ranked>>;
    fn explain(&self, bvid: &str) -> Result<Option<ScoreBreakdown>>;
    fn filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>>;
}

/// The profile and its history, moved between devices.
pub trait ProfileBackup: Send + Sync {
    fn profile(&self) -> InterestProfile;
    fn history(&self) -> Result<Vec<FeedbackEvent>>;
    /// Takes `profile` and `history` as they are: the profile already reflects the history,
    /// so the events are stored (and their marks restored) without being counted again.
    fn restore(&self, profile: InterestProfile, history: &[FeedbackEvent]) -> Result<()>;
}

/// The settings object as a whole.
pub trait SettingsPort: Send + Sync {
    fn settings(&self) -> Settings;
    /// Replaces the settings; invalid rules are rejected and nothing changes.
    fn set_settings(&self, settings: Settings) -> Result<()>;
    /// Changes the keys in `patch` (a JSON merge patch: objects merge key by key, other values
    /// replace, `null` restores the default) and keeps the rest as they are now.
    fn patch_json(&self, patch: &str) -> Result<()>;
}

/// The play lists, moved between devices.
pub trait PlaylistBackup: Send + Sync {
    fn export_lists(&self) -> Result<Vec<(String, Vec<VideoCard>)>>;
    /// Replaces every list at once.
    fn replace_all(&self, lists: &[(String, Vec<VideoCard>)]) -> Result<()>;
}

/// A video's card: the one shown already, else looked up.
#[async_trait]
pub trait VideoLookup: Send + Sync {
    async fn card(&self, bvid: &str) -> Result<VideoCard>;
}

/// A video's streams at a quality.
#[async_trait]
pub trait VideoStreams: VideoLookup {
    async fn playback(&self, bvid: &str, qn: u32) -> Result<Playback>;
}

/// The recommendation pool as the 竖屏推流 picks from it.
#[async_trait]
pub trait PortraitPool: Send + Sync {
    /// Up to `n` pooled cards in ranked order, leaving out `skip`. Nothing is recorded as shown.
    fn ranked(&self, n: usize, skip: &HashSet<String>) -> Result<Vec<VideoCard>>;
    /// Fetches a new batch into the pool.
    async fn fetch_more(&self) -> Result<()>;
}

/// Whether a video is portrait.
#[async_trait]
pub trait ShapeLookup: Send + Sync {
    /// As the lists it was shown in said; None when they did not.
    fn known_portrait(&self, bvid: &str) -> Option<bool>;
    /// From its details: one request.
    async fn look_up_portrait(&self, bvid: &str) -> Result<bool>;
}

pub trait LoginState: Send + Sync {
    fn is_logged_in(&self) -> bool;
}

/// Drops the cards the rule table blocks; also says how many went.
pub(crate) fn keep_cards(rules: &dyn RuleSource, cards: Vec<VideoCard>) -> Result<(Vec<VideoCard>, usize)> {
    let pipeline = rules.card_filter();
    let before = cards.len();
    let kept: Vec<VideoCard> = cards.into_iter().filter(|c| !pipeline.judge(c).is_blocked()).collect();
    let hidden = before - kept.len();
    Ok((kept, hidden))
}

/// A watched lookup over a set read in one go.
pub(crate) struct WatchedSet(pub HashSet<String>);

impl rubi_filter::WatchedLookup for WatchedSet {
    fn is_watched(&self, bvid: &str) -> bool {
        self.0.contains(bvid)
    }
}

/// What plays after a video when nothing else says: the recommended videos, read by id.
#[async_trait::async_trait]
pub trait RelatedSource: Send + Sync {
    async fn related(&self, bvid: &str) -> Vec<VideoCard>;
}
