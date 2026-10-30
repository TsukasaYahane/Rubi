use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{SourceKind, VideoCard};
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{card_of, card_of_dynamic, str_of};

const HOME_RCMD: &str = "https://api.bilibili.com/x/web-interface/wbi/index/top/feed/rcmd";
const POPULAR: &str = "https://api.bilibili.com/x/web-interface/popular";
const RANKING: &str = "https://api.bilibili.com/x/web-interface/ranking/v2";
const RELATED: &str = "https://api.bilibili.com/x/web-interface/archive/related";
const REGION: &str = "https://api.bilibili.com/x/web-interface/dynamic/region";
const DYNAMIC_FEED: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/all";

/// Lists that feed the candidate pool. Cards come back unfiltered, ads included.
#[async_trait]
pub trait FeedApi: Send + Sync {
    async fn home_rcmd(&self, fresh_idx: u32, ps: u32) -> Result<Vec<VideoCard>>;
    async fn popular(&self, pn: u32, ps: u32) -> Result<Vec<VideoCard>>;
    async fn ranking(&self, rid: u32) -> Result<Vec<VideoCard>>;
    async fn related(&self, bvid: &str) -> Result<Vec<VideoCard>>;
    async fn zone_newest(&self, rid: u32, pn: u32, ps: u32) -> Result<Vec<VideoCard>>;
    /// Videos from followed UPs (login required). Returns the next page offset.
    async fn followed_videos(&self, offset: Option<&str>) -> Result<(Vec<VideoCard>, Option<String>)>;
}

pub struct WebFeed {
    http: Arc<HttpClient>,
}

impl WebFeed {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

fn cards(list: &Value, source: SourceKind) -> Vec<VideoCard> {
    list.as_array().map(|a| a.iter().map(|v| card_of(v, source)).collect()).unwrap_or_default()
}

#[async_trait]
impl FeedApi for WebFeed {
    async fn home_rcmd(&self, fresh_idx: u32, ps: u32) -> Result<Vec<VideoCard>> {
        let p = params![
            "web_location" => 1430650, "feed_version" => "V8", "homepage_ver" => 1, "y_num" => 4,
            "fresh_type" => 4, "fresh_idx" => fresh_idx, "fresh_idx_1h" => fresh_idx, "brush" => fresh_idx, "ps" => ps,
        ];
        let data = self.http.get_json(HOME_RCMD, p, true).await?;
        Ok(cards(&data["item"], SourceKind::BiliRecommend))
    }

    async fn popular(&self, pn: u32, ps: u32) -> Result<Vec<VideoCard>> {
        let data = self.http.get_json(POPULAR, params!["pn" => pn, "ps" => ps], false).await?;
        Ok(cards(&data["list"], SourceKind::Popular))
    }

    async fn ranking(&self, rid: u32) -> Result<Vec<VideoCard>> {
        let data = self.http.get_json(RANKING, params!["rid" => rid, "type" => "all"], false).await?;
        Ok(cards(&data["list"], SourceKind::Ranking))
    }

    async fn related(&self, bvid: &str) -> Result<Vec<VideoCard>> {
        let data = self.http.get_json(RELATED, params!["bvid" => bvid], false).await?;
        Ok(cards(&data, SourceKind::Related))
    }

    async fn zone_newest(&self, rid: u32, pn: u32, ps: u32) -> Result<Vec<VideoCard>> {
        let data = self.http.get_json(REGION, params!["rid" => rid, "pn" => pn, "ps" => ps], false).await?;
        Ok(cards(&data["archives"], SourceKind::PreferredZone))
    }

    async fn followed_videos(&self, offset: Option<&str>) -> Result<(Vec<VideoCard>, Option<String>)> {
        let mut p = params!["type" => "video", "timezone_offset" => -480];
        if let Some(o) = offset {
            p.push(("offset".into(), o.into()));
        }
        let data = self.http.get_json(DYNAMIC_FEED, p, false).await?;
        let items =
            data["items"].as_array().map(|a| a.iter().filter_map(card_of_dynamic).collect()).unwrap_or_default();
        let next =
            Some(str_of(&data["offset"])).filter(|o| !o.is_empty() && data["has_more"].as_bool().unwrap_or(false));
        Ok((items, next))
    }
}
