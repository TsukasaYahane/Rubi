//! 关注: followed UPs' videos and posts, and copying the following list into the profile.

use std::sync::Arc;

use rubi_api::{DynamicApi, FeedApi, LibraryApi};
use rubi_model::{Dynamic, FollowedUp, VideoCard, now_secs};

use crate::error::Result;
use crate::ports::{FeedbackSink, RuleSource, keep_cards};
use crate::videos::CardCache;

/// Following lists are fetched at most this many pages (50 per page).
const MAX_FOLLOWING_PAGES: u32 = 40;

/// A page of followed UPs' videos.
#[derive(Debug, Clone, Default)]
pub struct FollowedPage {
    pub cards: Vec<VideoCard>,
    pub next: Option<String>,
    pub hidden: usize,
}

pub struct Following {
    feed: Arc<dyn FeedApi>,
    dynamics: Arc<dyn DynamicApi>,
    library: Arc<dyn LibraryApi>,
    rules: Arc<dyn RuleSource>,
    feedback: Arc<dyn FeedbackSink>,
    cards: Arc<CardCache>,
}

impl Following {
    pub(crate) fn new(
        feed: Arc<dyn FeedApi>,
        dynamics: Arc<dyn DynamicApi>,
        library: Arc<dyn LibraryApi>,
        rules: Arc<dyn RuleSource>,
        feedback: Arc<dyn FeedbackSink>,
        cards: Arc<CardCache>,
    ) -> Self {
        Self { feed, dynamics, library, rules, feedback, cards }
    }

    /// One page of videos from followed UPs, filtered like every list.
    pub async fn videos(&self, offset: Option<&str>) -> Result<FollowedPage> {
        let (cards, next) = self.feed.followed_videos(offset).await?;
        let (cards, hidden) = keep_cards(self.rules.as_ref(), cards)?;
        self.cards.remember(&cards);
        Ok(FollowedPage { cards, next, hidden })
    }

    /// Followed UPs with recent posts, for the bar above the feed.
    pub async fn ups(&self) -> Result<Vec<FollowedUp>> {
        Ok(self.dynamics.followed_ups().await?)
    }

    /// One UP's posts of every kind. A post whose video the rules block is dropped; other
    /// kinds are kept. Also returns the next offset.
    pub async fn up_posts(&self, mid: u64, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        let (items, next) = self.dynamics.up_dynamics(mid, offset).await?;
        let pipeline = self.rules.rules().card_pipeline(None, now_secs)?;
        let kept: Vec<Dynamic> =
            items.into_iter().filter(|d| d.video.as_ref().is_none_or(|v| !pipeline.judge(v).is_blocked())).collect();
        let videos: Vec<VideoCard> = kept
            .iter()
            .flat_map(|d| d.video.iter().chain(d.orig.iter().flat_map(|o| o.video.iter())))
            .cloned()
            .collect();
        self.cards.remember(&videos);
        Ok((kept, next))
    }

    /// Copies the account's following list into the follow factor. Returns how many.
    pub async fn sync_to_profile(&self) -> Result<usize> {
        let mut all = Vec::new();
        for pn in 1..=MAX_FOLLOWING_PAGES {
            let (mids, total) = self.library.followings(pn).await?;
            let done = mids.is_empty();
            all.extend(mids);
            if done || all.len() as u32 >= total {
                break;
            }
        }
        all.sort_unstable();
        all.dedup();
        self.feedback.set_followed(&all)?;
        Ok(all.len())
    }
}
