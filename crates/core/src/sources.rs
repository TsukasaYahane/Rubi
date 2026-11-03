//! Adapters that turn [`FeedApi`] calls into recommendation [`Source`]s.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rubi_api::{ApiError, FeedApi};
use rubi_model::{SourceKind, VideoCard};
use rubi_recommend::{Source, SourceError};

use crate::feed::{Seedable, Seeds};

fn err(kind: SourceKind) -> impl Fn(ApiError) -> SourceError {
    move |e| SourceError { kind, message: e.to_string() }
}

/// B 站's own recommendation, used as one weighted source among many.
pub struct BiliRecommendSource {
    api: Arc<dyn FeedApi>,
    fresh_idx: AtomicU32,
}

impl BiliRecommendSource {
    pub fn new(api: Arc<dyn FeedApi>) -> Self {
        Self { api, fresh_idx: AtomicU32::new(1) }
    }
}

#[async_trait]
impl Source for BiliRecommendSource {
    fn kind(&self) -> SourceKind {
        SourceKind::BiliRecommend
    }

    async fn fetch(&self, want: usize) -> Result<Vec<VideoCard>, SourceError> {
        let idx = self.fresh_idx.fetch_add(1, Ordering::Relaxed);
        self.api.home_rcmd(idx, want.clamp(1, 30) as u32).await.map_err(err(self.kind()))
    }
}

pub struct PopularSource {
    api: Arc<dyn FeedApi>,
    page: AtomicU32,
}

impl PopularSource {
    pub fn new(api: Arc<dyn FeedApi>) -> Self {
        Self { api, page: AtomicU32::new(1) }
    }
}

#[async_trait]
impl Source for PopularSource {
    fn kind(&self) -> SourceKind {
        SourceKind::Popular
    }

    async fn fetch(&self, want: usize) -> Result<Vec<VideoCard>, SourceError> {
        // Popular has about 10 pages; cycle through them.
        let pn = (self.page.fetch_add(1, Ordering::Relaxed) - 1) % 10 + 1;
        self.api.popular(pn, want.clamp(1, 50) as u32).await.map_err(err(self.kind()))
    }
}

pub struct RankingSource {
    api: Arc<dyn FeedApi>,
}

impl RankingSource {
    pub fn new(api: Arc<dyn FeedApi>) -> Self {
        Self { api }
    }
}

#[async_trait]
impl Source for RankingSource {
    fn kind(&self) -> SourceKind {
        SourceKind::Ranking
    }

    async fn fetch(&self, want: usize) -> Result<Vec<VideoCard>, SourceError> {
        let mut list = self.api.ranking(0).await.map_err(err(self.kind()))?;
        list.truncate(want);
        Ok(list)
    }
}

/// New uploads of followed UPs. Yields nothing for guests.
pub struct FollowedUpsSource {
    api: Arc<dyn FeedApi>,
    offset: Mutex<Option<String>>,
}

impl FollowedUpsSource {
    pub fn new(api: Arc<dyn FeedApi>) -> Self {
        Self { api, offset: Mutex::new(None) }
    }
}

#[async_trait]
impl Source for FollowedUpsSource {
    fn kind(&self) -> SourceKind {
        SourceKind::FollowedUps
    }

    async fn fetch(&self, _want: usize) -> Result<Vec<VideoCard>, SourceError> {
        let offset = self.offset.lock().unwrap_or_else(|e| e.into_inner()).clone();
        match self.api.followed_videos(offset.as_deref()).await {
            Ok((cards, next)) => {
                *self.offset.lock().unwrap_or_else(|e| e.into_inner()) = next;
                Ok(cards)
            }
            Err(ApiError::NotLoggedIn) => Ok(Vec::new()),
            Err(e) => Err(err(self.kind())(e)),
        }
    }
}

/// Videos related to what the user recently liked or finished.
pub struct RelatedSource {
    api: Arc<dyn FeedApi>,
    seeds: Mutex<Vec<String>>,
}

impl RelatedSource {
    pub fn new(api: Arc<dyn FeedApi>) -> Self {
        Self { api, seeds: Mutex::new(Vec::new()) }
    }
}

impl Seedable for RelatedSource {
    fn seed(&self, seeds: &Seeds) {
        *self.seeds.lock().unwrap_or_else(|e| e.into_inner()) = seeds.liked.clone();
    }
}

#[async_trait]
impl Source for RelatedSource {
    fn kind(&self) -> SourceKind {
        SourceKind::Related
    }

    async fn fetch(&self, want: usize) -> Result<Vec<VideoCard>, SourceError> {
        let seeds: Vec<String> = self.seeds.lock().unwrap_or_else(|e| e.into_inner()).iter().take(3).cloned().collect();
        let mut out = Vec::new();
        for s in seeds {
            out.extend(self.api.related(&s).await.map_err(err(self.kind()))?);
            if out.len() >= want {
                break;
            }
        }
        Ok(out)
    }
}

/// Recommended videos of the v2 main regions the profile likes most (mapped by `zones`).
pub struct PreferredZoneSource {
    api: Arc<dyn FeedApi>,
    zones: Mutex<Vec<u32>>,
}

impl PreferredZoneSource {
    pub fn new(api: Arc<dyn FeedApi>) -> Self {
        Self { api, zones: Mutex::new(Vec::new()) }
    }
}

impl Seedable for PreferredZoneSource {
    fn seed(&self, seeds: &Seeds) {
        *self.zones.lock().unwrap_or_else(|e| e.into_inner()) = seeds.zones.clone();
    }
}

#[async_trait]
impl Source for PreferredZoneSource {
    fn kind(&self) -> SourceKind {
        SourceKind::PreferredZone
    }

    async fn fetch(&self, want: usize) -> Result<Vec<VideoCard>, SourceError> {
        let zones: Vec<u32> = self.zones.lock().unwrap_or_else(|e| e.into_inner()).iter().take(3).copied().collect();
        let per = (want / zones.len().max(1)).clamp(5, 30) as u32;
        // A region the endpoint refuses is skipped; the source fails only when every one does.
        let mut out = Vec::new();
        let mut last_err = None;
        for z in &zones {
            match self.api.region_feed(*z, 1, per).await {
                Ok(cards) => out.extend(cards),
                Err(e) => last_err = Some(e),
            }
        }
        match last_err {
            Some(e) if out.is_empty() && !zones.is_empty() => Err(err(self.kind())(e)),
            _ => Ok(out),
        }
    }
}
