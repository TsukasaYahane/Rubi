//! What services need from each other, as narrow traits: the rule table to filter with, and
//! somewhere to send feedback. [`FeedService`] provides both; services never see it whole.

use rubi_filter::RuleSet;
use rubi_model::{FeedbackEvent, VideoCard, now_secs};

use crate::error::Result;
use crate::feed::{AppStore, FeedService};

/// The current rule table.
pub trait RuleSource: Send + Sync {
    fn rules(&self) -> RuleSet;
}

/// Where user actions go to shape the recommendation profile.
pub trait FeedbackSink: Send + Sync {
    fn record(&self, ev: FeedbackEvent) -> Result<()>;
    fn set_followed(&self, mids: &[u64]) -> Result<()>;
}

impl<S: AppStore> RuleSource for FeedService<S> {
    fn rules(&self) -> RuleSet {
        self.settings().rules
    }
}

impl<S: AppStore> FeedbackSink for FeedService<S> {
    fn record(&self, ev: FeedbackEvent) -> Result<()> {
        FeedService::record(self, ev)
    }

    fn set_followed(&self, mids: &[u64]) -> Result<()> {
        FeedService::set_followed(self, mids)
    }
}

/// Drops the cards the rule table blocks; also says how many went.
pub(crate) fn keep_cards(rules: &dyn RuleSource, cards: Vec<VideoCard>) -> Result<(Vec<VideoCard>, usize)> {
    let pipeline = rules.rules().card_pipeline(None, now_secs)?;
    let before = cards.len();
    let kept: Vec<VideoCard> = cards.into_iter().filter(|c| !pipeline.judge(c).is_blocked()).collect();
    let hidden = before - kept.len();
    Ok((kept, hidden))
}
