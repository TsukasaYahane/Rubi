//! Videos: looking one up, its streams, and the cache of cards already handed out.

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::join;
use rubi_api::{Page, PlayUrl, Season, VideoApi};
use rubi_model::{VideoCard, VideoRef, find_video_ref};

use crate::error::{CoreError, Result};
use crate::ports::{ShapeLookup, VideoLookup, VideoStreams};
use crate::recent::Recent;

/// Cards kept so actions and signals on a shown video need no second lookup.
const CARD_CACHE: usize = 3_000;

/// Highest qn a non-member account may play (1080P). Above it (1080P+, 1080P60, 4K, HDR,
/// Dolby, 8K) is 大会员 only, which Rubi does not offer.
pub const MAX_FREE_QN: u32 = 80;

/// Cards seen in any list, by BV id. Shared by every service that hands cards out. Kept without
/// the raw API JSON they were parsed from: the filters have judged them already, and the raw
/// object is most of a card's size.
pub struct CardCache {
    cards: Recent<String, VideoCard>,
    /// Whether each video is portrait, where its list said (read before the raw JSON goes).
    portrait: Recent<String, bool>,
}

impl Default for CardCache {
    fn default() -> Self {
        Self { cards: Recent::new(CARD_CACHE), portrait: Recent::new(CARD_CACHE) }
    }
}

impl CardCache {
    pub fn remember(&self, cards: &[VideoCard]) {
        for c in cards {
            if let Some(p) = portrait_of(&c.raw) {
                self.portrait.put(c.bvid.clone(), p);
            }
            let mut slim = c.clone();
            slim.raw = serde_json::Value::Null;
            self.cards.put(c.bvid.clone(), slim);
        }
    }

    /// Whether `bvid` is a portrait video, when a list it was shown in said so.
    pub fn portrait(&self, bvid: &str) -> Option<bool> {
        self.portrait.get(&bvid.to_owned())
    }

    /// Records a video's shape learned some other way (its details).
    fn note_portrait(&self, bvid: &str, portrait: bool) {
        self.portrait.put(bvid.to_owned(), portrait);
    }

    /// Drops every card; the next action on one looks it up again.
    fn clear(&self) {
        self.cards.clear();
        self.portrait.clear();
    }

    fn get(&self, bvid: &str) -> Option<VideoCard> {
        self.cards.get(&bvid.to_owned())
    }
}

/// Whether a video is portrait (taller than wide), from the `dimension` B站 gives with it
/// (`rotate` 1: width and height are swapped). `None` when the JSON has no size.
pub fn portrait_of(raw: &serde_json::Value) -> Option<bool> {
    let d = &raw["dimension"];
    let (w, h) = (d["width"].as_u64()?, d["height"].as_u64()?);
    if w == 0 || h == 0 {
        return None;
    }
    Some(if d["rotate"].as_u64() == Some(1) { w > h } else { h > w })
}

/// What the player page shows about a video besides its card: its parts (分P) and the
/// 合集 it belongs to.
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub card: VideoCard,
    pub parts: Vec<Page>,
    pub season: Option<Season>,
}

/// Where watching can pick up again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumePoint {
    pub cid: u64,
    pub position_ms: u64,
}

/// Positions this close to the start are not worth offering.
const RESUME_MIN_MS: u64 = 10_000;
/// Nor ones this close to the end (the video was finished).
const RESUME_END_MS: u64 = 15_000;

/// Whether a stopped position is worth offering for a part `duration_s` long (0: unknown).
fn worth_resuming(position_ms: u64, duration_s: u32) -> bool {
    let end = u64::from(duration_s) * 1000;
    position_ms >= RESUME_MIN_MS && (end == 0 || position_ms + RESUME_END_MS < end)
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
        let (view, tags) = join(self.api.view(bvid), self.api.tags(bvid)).await;
        let mut card = view?.card;
        card.tags = tags.unwrap_or_default();
        self.cards.remember(std::slice::from_ref(&card));
        Ok(card)
    }

    /// Forgets the cards shown so far (after a video chain is left, or when memory is low).
    pub fn forget_shown(&self) {
        self.cards.clear();
    }

    /// Whether `bvid` is a portrait video, when a list it was shown in said so.
    pub fn known_portrait(&self, bvid: &str) -> Option<bool> {
        self.cards.portrait(bvid)
    }

    /// Whether `bvid` is a portrait video: from the lists when they said, else from its
    /// details (one request; only the shape is kept, the card is looked up in full later).
    pub async fn portrait(&self, bvid: &str) -> Result<bool> {
        if let Some(p) = self.cards.portrait(bvid) {
            return Ok(p);
        }
        let p = portrait_of(&self.api.view(bvid).await?.card.raw).unwrap_or(false);
        self.cards.note_portrait(bvid, p);
        Ok(p)
    }

    /// Fresh details for the player page: the card (with tags and cid), its parts and 合集.
    pub async fn detail(&self, bvid: &str) -> Result<VideoInfo> {
        let (detail, tags) = join(self.api.view(bvid), self.api.tags(bvid)).await;
        let detail = detail?;
        let mut card = detail.card;
        card.tags = tags.unwrap_or_default();
        card.cid = card.cid.or(Some(detail.cid)).filter(|&c| c > 0);
        self.cards.remember(std::slice::from_ref(&card));
        Ok(VideoInfo { card, parts: detail.pages, season: detail.season })
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

    /// Where the signed-in user stopped watching `bvid`, when it is worth offering (not the
    /// first seconds, not the very end). Guests and failures get `None`: it is only a hint.
    pub async fn resume_point(&self, bvid: &str) -> Option<ResumePoint> {
        let card = self.card(bvid).await.ok()?;
        let cid = match card.cid {
            Some(c) => c,
            None => self.api.view(bvid).await.ok()?.cid,
        };
        let (cid, position_ms) = self.api.last_watched(bvid, cid).await.ok()??;
        // Multi-part durations are per part; the card's total still bounds the end.
        worth_resuming(position_ms, card.duration_s).then_some(ResumePoint { cid, position_ms })
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

    /// 投屏: the video (part `cid`, or the first) as one file, at most the free quality, with the
    /// headers it must be fetched with (the phone relays it to the TV with them).
    pub async fn cast_file(
        &self,
        bvid: &str,
        cid: Option<u64>,
        qn: u32,
    ) -> Result<(rubi_api::SingleFile, Vec<(String, String)>)> {
        let cid = match cid {
            Some(c) => c,
            None => match self.cards.get(bvid).and_then(|c| c.cid) {
                Some(c) => c,
                None => self.api.view(bvid).await?.cid,
            },
        };
        let mut f = self.api.single_file(bvid, cid, qn.min(MAX_FREE_QN)).await?;
        f.accept.retain(|(qn, _)| *qn <= MAX_FREE_QN);
        Ok((f, self.api.stream_headers()))
    }

    /// Streams of one part (`cid`), e.g. a branch of an interactive video.
    pub async fn playback_part(&self, bvid: &str, cid: u64, qn: u32) -> Result<Playback> {
        let mut url = self.api.playurl(bvid, cid, qn.min(MAX_FREE_QN)).await?;
        url.accept.retain(|(qn, _)| *qn <= MAX_FREE_QN);
        Ok(Playback { cid, url, headers: self.api.stream_headers() })
    }
}

#[async_trait]
impl ShapeLookup for Videos {
    fn known_portrait(&self, bvid: &str) -> Option<bool> {
        Videos::known_portrait(self, bvid)
    }

    async fn look_up_portrait(&self, bvid: &str) -> Result<bool> {
        self.portrait(bvid).await
    }
}

#[async_trait]
impl VideoLookup for Videos {
    async fn card(&self, bvid: &str) -> Result<VideoCard> {
        Videos::card(self, bvid).await
    }
}

#[async_trait]
impl VideoStreams for Videos {
    async fn playback(&self, bvid: &str, qn: u32) -> Result<Playback> {
        Videos::playback(self, bvid, qn).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_follows_dimension_and_rotation() {
        let d = |w: u64, h: u64, r: u64| serde_json::json!({ "dimension": { "width": w, "height": h, "rotate": r } });
        assert_eq!(portrait_of(&d(1080, 1920, 0)), Some(true));
        assert_eq!(portrait_of(&d(1920, 1080, 0)), Some(false));
        assert_eq!(portrait_of(&d(1920, 1080, 1)), Some(true), "rotated: the sides swap");
        assert_eq!(portrait_of(&serde_json::json!({})), None);
    }

    #[test]
    fn resume_skips_the_start_and_the_end() {
        assert!(!worth_resuming(5_000, 600));
        assert!(worth_resuming(432_000, 760));
        assert!(!worth_resuming(750_000, 760), "finished");
        assert!(worth_resuming(60_000, 0), "unknown length");
    }
}
