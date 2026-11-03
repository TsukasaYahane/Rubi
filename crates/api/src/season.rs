//! 合集 subscriptions: subscribing to a season, the seasons subscribed to, and a season's videos.

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{SeasonSub, SourceKind, VideoCard};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{card_of, image_of, str_of, title_of, u64_of};

const SUBSCRIBE: &str = "https://api.bilibili.com/x/v3/fav/season/fav";
const UNSUBSCRIBE: &str = "https://api.bilibili.com/x/v3/fav/season/unfav";
const COLLECTED: &str = "https://api.bilibili.com/x/v3/fav/folder/collected/list";
const SEASON_VIDEOS: &str = "https://api.bilibili.com/x/polymer/web-space/seasons_archives_list";

/// `type` of a 合集 in the collected list (11 is someone else's favourite folder).
const TYPE_SEASON: u64 = 21;
/// Subscriptions per page, as the web asks.
const PAGE: u32 = 20;
/// Videos of a season per page.
pub const SEASON_PAGE: u32 = 30;

#[async_trait]
pub trait SeasonApi: Send + Sync {
    async fn subscribe(&self, season_id: u64, on: bool) -> Result<()>;
    /// Page `pn` (from 1) of the 合集 subscribed to; also whether more follow.
    async fn subscribed(&self, pn: u32) -> Result<(Vec<SeasonSub>, bool)>;
    /// Page `pn` (from 1) of a season's videos, in the season's order; also the season's size.
    async fn videos(&self, mid: u64, season_id: u64, pn: u32) -> Result<(Vec<VideoCard>, u32)>;
}

pub struct WebSeasons {
    http: Arc<HttpClient>,
}

impl WebSeasons {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl SeasonApi for WebSeasons {
    async fn subscribe(&self, season_id: u64, on: bool) -> Result<()> {
        let form = params!["season_id" => season_id, "platform" => "web"];
        self.http.post_form(if on { SUBSCRIBE } else { UNSUBSCRIBE }, form).await.map(|_| ())
    }

    async fn subscribed(&self, pn: u32) -> Result<(Vec<SeasonSub>, bool)> {
        let me: u64 = self.http.session().dede_user_id.and_then(|m| m.parse().ok()).ok_or(ApiError::NotLoggedIn)?;
        let d = self
            .http
            .get_json(COLLECTED, params!["pn" => pn, "ps" => PAGE, "up_mid" => me, "platform" => "web"], false)
            .await?;
        let list = d["list"]
            .as_array()
            .map(|a| a.iter().filter(|v| u64_of(&v["type"]) == TYPE_SEASON).map(sub_of).collect())
            .unwrap_or_default();
        Ok((list, d["has_more"].as_bool().unwrap_or(false)))
    }

    async fn videos(&self, mid: u64, season_id: u64, pn: u32) -> Result<(Vec<VideoCard>, u32)> {
        let p = params![
            "mid" => mid, "season_id" => season_id, "sort_reverse" => "false",
            "page_num" => pn, "page_size" => SEASON_PAGE,
        ];
        let d = self.http.get_json(SEASON_VIDEOS, p, true).await?;
        Ok(season_videos_of(&d, mid))
    }
}

pub(crate) fn sub_of(v: &Value) -> SeasonSub {
    SeasonSub {
        id: u64_of(&v["id"]),
        title: title_of(&v["title"]),
        cover: image_of(&v["cover"]),
        count: u64_of(&v["media_count"]) as u32,
        up_mid: u64_of(&v["upper"]["mid"]),
        up_name: str_of(&v["upper"]["name"]),
    }
}

/// A page of `seasons_archives_list`; the cards carry the season's owner.
pub(crate) fn season_videos_of(d: &Value, mid: u64) -> (Vec<VideoCard>, u32) {
    let cards = d["archives"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|v| {
                    let mut c = card_of(v, SourceKind::Related);
                    if c.owner.mid == 0 {
                        c.owner.mid = mid;
                    }
                    c
                })
                .collect()
        })
        .unwrap_or_default();
    let total = [&d["meta"]["total"], &d["page"]["total"]].into_iter().map(u64_of).find(|&n| n > 0).unwrap_or(0);
    (cards, total as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_subscription_and_a_season_page() {
        let v = serde_json::json!({
            "id": 7, "title": "技术周报", "cover": "//i0.hdslb.com/a.jpg", "media_count": 40, "type": 21,
            "upper": { "mid": 9, "name": "UP" }
        });
        let s = sub_of(&v);
        assert_eq!((s.id, s.count, s.up_mid, s.up_name.as_str()), (7, 40, 9, "UP"));
        assert!(s.cover.starts_with("https:"));

        let d = serde_json::json!({
            "archives": [{ "aid": 1, "bvid": "BV1", "title": "第 1 期", "pic": "https://x/1.jpg", "duration": 60, "stat": { "view": 5 } }],
            "meta": { "total": 40 }
        });
        let (cards, total) = season_videos_of(&d, 9);
        assert_eq!((cards[0].bvid.as_str(), cards[0].owner.mid, cards[0].stats.view, total), ("BV1", 9, 5, 40));
    }
}
