//! The home feed as the app sees it: pages of ranked cards, their explanations, and the
//! feedback that shapes them.

use std::sync::Arc;

use rubi_model::{DislikeReason, FeedbackEvent, ItemRef, Signal, now_secs};
use rubi_recommend::{Ranked, ScoreBreakdown};
use rubi_store::{FilterLogEntry, SqliteStore};

use crate::error::Result;
use crate::feed::{FeedService, RefreshReport};
use crate::videos::{CardCache, Videos};

pub struct Home {
    feed: Arc<FeedService<SqliteStore>>,
    videos: Arc<Videos>,
    cards: Arc<CardCache>,
}

impl Home {
    pub(crate) fn new(feed: Arc<FeedService<SqliteStore>>, videos: Arc<Videos>, cards: Arc<CardCache>) -> Self {
        Self { feed, videos, cards }
    }

    /// Fills the candidate pool from every source that is not cooling down.
    pub async fn refresh(&self, per_source: usize) -> Result<RefreshReport> {
        self.feed.refresh(per_source).await
    }

    /// Call before rebuilding the list (pull to refresh) so earlier cards may come back.
    pub fn new_session(&self) {
        self.feed.new_session();
    }

    /// The next `n` cards not yet shown since [`new_session`](Self::new_session).
    pub fn next_page(&self, n: usize) -> Result<Vec<Ranked>> {
        let ranked = self.feed.next_page(n)?;
        self.cards.remember(&ranked.iter().map(|r| r.card.clone()).collect::<Vec<_>>());
        Ok(ranked)
    }

    pub fn explain(&self, bvid: &str) -> Result<Option<ScoreBreakdown>> {
        self.feed.explain(bvid)
    }

    /// Records what the user did with a video. A tag dislike without a tag means the video's
    /// first tag.
    pub async fn record(&self, bvid: &str, signal: Signal) -> Result<()> {
        let card = self.videos.card(bvid).await?;
        let signal = match signal {
            Signal::Dislike { reason: DislikeReason::Tag(t) } if t.is_empty() => {
                Signal::Dislike { reason: DislikeReason::Tag(card.tags.first().cloned().unwrap_or_default()) }
            }
            other => other,
        };
        self.feed.record(FeedbackEvent { signal, item: Some(ItemRef::of(&card)), at: now_secs() })
    }

    /// What the rules removed lately, newest first.
    pub fn filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>> {
        self.feed.filter_log(limit)
    }
}
