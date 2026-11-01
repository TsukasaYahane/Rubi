//! Videos: looking one up, its streams, and the cache of cards already handed out.

use std::sync::Arc;

use rubi_api::{PlayUrl, VideoApi};
use rubi_model::{VideoCard, VideoRef, find_video_ref};

use crate::error::{CoreError, Result};
use crate::recent::Recent;

/// Cards kept so actions and signals on a shown video need no second lookup.
const CARD_CACHE: usize = 3_000;

/// Highest qn a non-member account may play (1080P). Above it (1080P+, 1080P60, 4K, HDR,
/// Dolby, 8K) is 大会员 only, which Rubi does not offer.
pub const MAX_FREE_QN: u32 = 80;

/// Cards seen in any list, by BV id. Shared by every service that hands cards out.
pub struct CardCache(Recent<String, VideoCard>);

impl Default for CardCache {
    fn default() -> Self {
        Self(Recent::new(CARD_CACHE))
    }
}

impl CardCache {
    pub fn remember(&self, cards: &[VideoCard]) {
        for c in cards {
            self.0.put(c.bvid.clone(), c.clone());
        }
    }

    /// Drops every card; the next action on one looks it up again.
    fn clear(&self) {
        self.0.clear();
    }

    fn get(&self, bvid: &str) -> Option<VideoCard> {
        self.0.get(&bvid.to_owned())
    }
}

/// Streams of one part, with only the qualities Rubi offers.
#[derive(Debug, Clone)]
pub struct Playback {
    pub cid: u64,
    pub url: PlayUrl,
    /// Headers every stream request must carry (no cookies).
    pub headers: Vec<(String, String)>,
}

pub struct Videos {
    api: Arc<dyn VideoApi>,
    cards: Arc<CardCache>,
}

impl Videos {
    pub(crate) fn new(api: Arc<dyn VideoApi>, cards: Arc<CardCache>) -> Self {
        Self { api, cards }
    }

    /// The card for `bvid`: the one already shown, else looked up (with tags).
    pub async fn card(&self, bvid: &str) -> Result<VideoCard> {
        if let Some(c) = self.cards.get(bvid) {
            return Ok(c);
        }
        let mut card = self.api.view(bvid).await?.card;
        card.tags = self.api.tags(bvid).await.unwrap_or_default();
        self.cards.remember(std::slice::from_ref(&card));
        Ok(card)
    }

    /// Forgets the cards shown so far (after a video chain is left, or when memory is low).
    pub fn forget_shown(&self) {
        self.cards.clear();
    }

    /// Fresh details for the player page: the card (with tags and cid) and how many parts.
    pub async fn detail(&self, bvid: &str) -> Result<(VideoCard, usize)> {
        let detail = self.api.view(bvid).await?;
        let mut card = detail.card;
        card.tags = self.api.tags(bvid).await.unwrap_or_default();
        card.cid = card.cid.or(Some(detail.cid)).filter(|&c| c > 0);
        self.cards.remember(std::slice::from_ref(&card));
        Ok((card, detail.pages.len().max(1)))
    }

    /// The BV id a video pointer stands for: av numbers are looked up, short links followed
    /// (one hop, the target page itself is never fetched).
    pub async fn resolve(&self, video: VideoRef) -> Result<String> {
        match video {
            VideoRef::Bvid(bvid) => Ok(bvid),
            VideoRef::Aid(aid) => Ok(self.api.bvid_of(aid).await?),
            VideoRef::Short(url) => {
                let target = self.api.expand_short(&url).await?.unwrap_or_default();
                match find_video_ref(&target) {
                    Some(VideoRef::Bvid(bvid)) => Ok(bvid),
                    Some(VideoRef::Aid(aid)) => Ok(self.api.bvid_of(aid).await?),
                    _ => Err(CoreError::Invalid(format!(
                        "短链指向的不是视频：{}",
                        if target.is_empty() { &url } else { &target }
                    ))),
                }
            }
        }
    }

    /// Streams of the video's first part at `qn` (the server caps it to what the account may play).
    pub async fn playback(&self, bvid: &str, qn: u32) -> Result<Playback> {
        // Shown cards usually carry the cid already; only look it up when they do not.
        let cid = match self.cards.get(bvid).and_then(|c| c.cid) {
            Some(cid) => cid,
            None => self.api.view(bvid).await?.cid,
        };
        self.playback_part(bvid, cid, qn).await
    }

    /// Streams of one part (`cid`), e.g. a branch of an interactive video.
    pub async fn playback_part(&self, bvid: &str, cid: u64, qn: u32) -> Result<Playback> {
        let mut url = self.api.playurl(bvid, cid, qn.min(MAX_FREE_QN)).await?;
        url.accept.retain(|(qn, _)| *qn <= MAX_FREE_QN);
        Ok(Playback { cid, url, headers: self.api.stream_headers() })
    }
}
