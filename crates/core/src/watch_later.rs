//! 稍后再看: the list, and taking videos out of it. Adding is the video page's (`Actions`).

use std::sync::Arc;

use rubi_api::WatchLaterApi;
use rubi_model::HistoryEntry;

use crate::error::Result;
use crate::videos::CardCache;

pub struct WatchLater {
    api: Arc<dyn WatchLaterApi>,
    cards: Arc<CardCache>,
}

impl WatchLater {
    pub(crate) fn new(api: Arc<dyn WatchLaterApi>, cards: Arc<CardCache>) -> Self {
        Self { api, cards }
    }

    /// The list, with how far each was watched.
    pub async fn list(&self) -> Result<Vec<HistoryEntry>> {
        let items = self.api.list().await?;
        self.cards.remember(&items.iter().map(|h| h.card.clone()).collect::<Vec<_>>());
        Ok(items)
    }

    /// Takes videos (by aid) out, one request each, as B站 removes them.
    pub async fn remove(&self, aids: &[u64]) -> Result<()> {
        for &aid in aids {
            self.api.remove(aid).await?;
        }
        Ok(())
    }

    /// Takes the videos watched to the end out.
    pub async fn clear_watched(&self) -> Result<()> {
        Ok(self.api.clear_watched().await?)
    }
}
