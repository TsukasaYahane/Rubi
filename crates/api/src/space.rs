//! An UP's space: profile card and uploads.

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{Owner, SourceKind, Stats, UpProfile, VideoCard};
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{duration_of, str_of, u64_of};

const CARD: &str = "https://api.bilibili.com/x/web-interface/card";
const ARC_SEARCH: &str = "https://api.bilibili.com/x/space/wbi/arc/search";
const PAGE_SIZE: u32 = 30;

/// Browser fingerprint fields `arc/search` checks before answering (-352 without them).
const DM_IMG_STR: &str = "V2ViR0wgMS4wIChPcGVuR0wgRVMgMi4wIENocm9taXVtKQ";
const DM_COVER_IMG_STR: &str = "QU5HTEUgKEludGVsLCBJbnRlbChSKSBVSEQgR3JhcGhpY3MgNjMwICgweDAwMDAzRTlCKSBEaXJlY3QzRDExIHZzXzVfMCBwc181XzAsIEQzRDExKUdvb2dsZSBJbmMuIChJbnRlbC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpOrder {
    /// 最新发布
    Pubdate,
    /// 最多播放
    Click,
    /// 最多收藏
    Stow,
}

impl UpOrder {
    fn param(self) -> &'static str {
        match self {
            UpOrder::Pubdate => "pubdate",
            UpOrder::Click => "click",
            UpOrder::Stow => "stow",
        }
    }
}

/// One page of uploads.
#[derive(Debug, Clone, Default)]
pub struct UpVideos {
    pub items: Vec<UpVideo>,
    pub total: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone)]
/// `charging` marks 充电专属 videos (dropped by the core).
pub struct UpVideo {
    pub card: VideoCard,
    pub charging: bool,
}

#[async_trait]
pub trait SpaceApi: Send + Sync {
    async fn profile(&self, mid: u64) -> Result<UpProfile>;
    /// Page `pn` (from 1) of the UP's uploads, 30 per page.
    async fn videos(&self, mid: u64, pn: u32, order: UpOrder) -> Result<UpVideos>;
}

pub struct WebSpace {
    http: Arc<HttpClient>,
}

impl WebSpace {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

pub(crate) fn profile_of(d: &Value) -> UpProfile {
    let c = &d["card"];
    UpProfile {
        mid: u64_of(&c["mid"]),
        name: str_of(&c["name"]),
        face: str_of(&c["face"]),
        sign: str_of(&c["sign"]),
        level: u64_of(&c["level_info"]["current_level"]) as u32,
        official: [&c["Official"]["title"], &c["official_verify"]["desc"]]
            .into_iter()
            .map(str_of)
            .find(|s| !s.is_empty())
            .unwrap_or_default(),
        fans: [&d["follower"], &c["fans"]].into_iter().map(u64_of).max().unwrap_or(0),
        followings: u64_of(&c["attention"]),
        archives: u64_of(&d["archive_count"]),
        followed: d["following"].as_bool().unwrap_or(false),
    }
}

pub(crate) fn up_video_of(v: &Value) -> UpVideo {
    let card = VideoCard {
        bvid: str_of(&v["bvid"]),
        aid: u64_of(&v["aid"]),
        title: str_of(&v["title"]),
        desc: str_of(&v["description"]),
        cover: str_of(&v["pic"]),
        owner: Owner { mid: u64_of(&v["mid"]), name: str_of(&v["author"]) },
        duration_s: duration_of(&v["length"]),
        pubdate: u64_of(&v["created"]) as i64,
        stats: Stats { view: u64_of(&v["play"]), danmaku: u64_of(&v["video_review"]), ..Default::default() },
        sources: vec![SourceKind::Subscription],
        raw: v.clone(),
        ..Default::default()
    };
    let charging = v["is_charging_arc"].as_bool().unwrap_or(false) || u64_of(&v["elec_arc_type"]) > 0;
    UpVideo { card, charging }
}

#[async_trait]
impl SpaceApi for WebSpace {
    async fn profile(&self, mid: u64) -> Result<UpProfile> {
        let d = self.http.get_json(CARD, params!["mid" => mid, "photo" => "false"], false).await?;
        Ok(profile_of(&d))
    }

    async fn videos(&self, mid: u64, pn: u32, order: UpOrder) -> Result<UpVideos> {
        let p = params![
            "mid" => mid,
            "pn" => pn.max(1),
            "ps" => PAGE_SIZE,
            "order" => order.param(),
            "platform" => "web",
            "dm_img_list" => "[]",
            "dm_img_str" => DM_IMG_STR,
            "dm_cover_img_str" => DM_COVER_IMG_STR,
            "dm_img_inter" => r#"{"ds":[],"wh":[0,0,0],"of":[0,0,0]}"#,
        ];
        let d = self.http.get_json(ARC_SEARCH, p, true).await?;
        let items: Vec<UpVideo> =
            d["list"]["vlist"].as_array().map(|a| a.iter().map(up_video_of).collect()).unwrap_or_default();
        let total = u64_of(&d["page"]["count"]) as u32;
        let has_more = !items.is_empty() && pn.max(1) * PAGE_SIZE < total;
        Ok(UpVideos { items, total, has_more })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_profile() {
        let d = json!({"card": {"mid": "7", "name": "up", "sign": "s", "attention": 3, "fans": 10,
            "level_info": {"current_level": 6}, "Official": {"title": "知名 UP 主"}},
            "follower": 12, "archive_count": 40, "following": true});
        let p = profile_of(&d);
        assert_eq!((p.mid, p.level, p.fans, p.followings, p.archives, p.followed), (7, 6, 12, 3, 40, true));
        assert_eq!(p.official, "知名 UP 主");
    }

    #[test]
    fn parses_upload_and_charging_flag() {
        let v = json!({"aid": 1, "bvid": "BV1", "title": "t", "author": "up", "mid": 7, "length": "03:20",
            "play": 99, "video_review": 5, "created": 100, "is_charging_arc": true});
        let u = up_video_of(&v);
        assert_eq!((u.card.duration_s, u.card.stats.view, u.card.owner.mid), (200, 99, 7));
        assert!(u.charging);
        assert!(!up_video_of(&json!({"play": "--"})).charging);
    }
}
