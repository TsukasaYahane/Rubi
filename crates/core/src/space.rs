//! An UP's space: profile, uploads (充电专属 videos left out) and 专栏.

use std::sync::Arc;

use rubi_api::{SpaceApi, UpArticles, UpOrder};
use rubi_model::{UpProfile, VideoCard};

use crate::error::Result;
use crate::videos::CardCache;

/// A page of [`Space::uploads`].
#[derive(Debug, Clone, Default)]
pub struct UpUploads {
    pub items: Vec<VideoCard>,
    pub total: u32,
    pub hidden_charging: usize,
    pub has_more: bool,
}

pub struct Space {
    api: Arc<dyn SpaceApi>,
    cards: Arc<CardCache>,
}

impl Space {
    pub(crate) fn new(api: Arc<dyn SpaceApi>, cards: Arc<CardCache>) -> Self {
        Self { api, cards }
    }

    pub async fn profile(&self, mid: u64) -> Result<UpProfile> {
        Ok(self.api.profile(mid).await?)
    }

    /// Page `pn` (from 1) of an UP's 专栏, newest first.
    pub async fn articles(&self, mid: u64, pn: u32) -> Result<UpArticles> {
        Ok(self.api.articles(mid, pn.max(1)).await?)
    }

    /// Page `pn` (from 1) of an UP's uploads without 充电专属 videos.
    pub async fn uploads(&self, mid: u64, pn: u32, order: UpOrder) -> Result<UpUploads> {
        self.search_uploads(mid, pn, order, "").await
    }

    /// Like [`uploads`](Self::uploads), only those matching `keyword` (空间内搜索).
    pub async fn search_uploads(&self, mid: u64, pn: u32, order: UpOrder, keyword: &str) -> Result<UpUploads> {
        let page = self.api.videos(mid, pn.max(1), order, keyword.trim()).await?;
        let fetched = page.items.len();
        let items: Vec<VideoCard> = page.items.into_iter().filter(|v| !v.charging).map(|v| v.card).collect();
        self.cards.remember(&items);
        Ok(UpUploads { hidden_charging: fetched - items.len(), has_more: page.has_more, items, total: page.total })
    }
}
