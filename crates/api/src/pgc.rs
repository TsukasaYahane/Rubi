//! 番剧 / 电影 / 纪录片 / 电视剧 (PGC): the release timeline, the index with its filters, a
//! season's episodes, and an episode's streams with what the account may watch. Whether an
//! episode plays is the server's call; nothing here works around it.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{image_of, str_of, u64_of};
use crate::video::{PlayUrl, play_url_of};

const TIMELINE: &str = "https://api.bilibili.com/pgc/web/timeline";
const CONDITION: &str = "https://api.bilibili.com/pgc/season/index/condition";
const INDEX: &str = "https://api.bilibili.com/pgc/season/index/result";
const SEASON: &str = "https://api.bilibili.com/pgc/view/web/season";
const PLAYURL: &str = "https://api.bilibili.com/pgc/player/web/v2/playurl";
const INDEX_PAGE: u32 = 20;

/// One episode in the release timeline.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcRelease {
    pub season_id: u64,
    pub title: String,
    pub cover: String,
    /// 「第11话」.
    pub index: String,
    /// 「10:00」.
    pub time: String,
    pub published: bool,
}

/// A day of the release timeline.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcDay {
    /// 「11-3」.
    pub date: String,
    pub today: bool,
    pub releases: Vec<PgcRelease>,
}

/// One filter of the index (地区, 风格, 付费…) and its choices as (value, name).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcFilter {
    pub field: String,
    pub name: String,
    pub values: Vec<(String, String)>,
}

/// The index's filters and sort orders (value, name).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcConditions {
    pub filters: Vec<PgcFilter>,
    pub orders: Vec<(String, String)>,
}

/// A season in the index.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcSeason {
    pub season_id: u64,
    pub title: String,
    pub subtitle: String,
    pub cover: String,
    /// 「9.6」, empty when unrated.
    pub score: String,
    /// 「大会员」「付费」「独家」… as the poster's corner shows; empty when none.
    pub badge: String,
    /// 「全11话」「更新至第5话」.
    pub index_show: String,
    /// What the sort is by: 「553.3万追番」, 「2026年10月上线」.
    pub order_text: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcEpisode {
    pub ep_id: u64,
    pub aid: u64,
    pub bvid: String,
    pub cid: u64,
    /// 「1」「第1话」.
    pub title: String,
    pub long_title: String,
    pub badge: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PgcSeasonDetail {
    pub season_id: u64,
    pub title: String,
    pub cover: String,
    pub evaluate: String,
    pub score: String,
    pub areas: Vec<String>,
    pub styles: Vec<String>,
    /// 「2011年04月14日」.
    pub pub_time: String,
    pub episodes: Vec<PgcEpisode>,
    /// Whether B站 lets this season be cached for offline viewing at all.
    pub allow_download: bool,
}

/// An episode's streams and what the server says about them.
#[derive(Debug, Clone, PartialEq)]
pub struct PgcPlay {
    pub url: Option<PlayUrl>,
    /// Only a trial clip: the account has no right to the whole episode.
    pub preview: bool,
    /// The streams are DRM-protected: only official clients may play them.
    pub drm: bool,
}

#[async_trait]
pub trait PgcApi: Send + Sync {
    /// The 番剧 / 国创 release timeline around today (`types` 1 番剧, 4 国创).
    async fn timeline(&self, types: u32) -> Result<Vec<PgcDay>>;
    async fn conditions(&self, season_type: u32) -> Result<PgcConditions>;
    /// Page `page` (from 1) of the index; `filters` as (field, value); also whether more follow.
    async fn index(
        &self,
        season_type: u32,
        filters: &[(String, String)],
        order: &str,
        page: u32,
    ) -> Result<(Vec<PgcSeason>, bool)>;
    async fn season(&self, season_id: u64) -> Result<PgcSeasonDetail>;
    async fn play(&self, ep_id: u64, qn: u32) -> Result<PgcPlay>;
    fn stream_headers(&self) -> Vec<(String, String)>;
}

pub struct WebPgc {
    http: Arc<HttpClient>,
}

impl WebPgc {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl PgcApi for WebPgc {
    async fn timeline(&self, types: u32) -> Result<Vec<PgcDay>> {
        let d = self.http.get_json(TIMELINE, params!["types" => types, "before" => 3, "after" => 6], false).await?;
        Ok(days_of(&d))
    }

    async fn conditions(&self, season_type: u32) -> Result<PgcConditions> {
        let d = self.http.get_json(CONDITION, params!["season_type" => season_type, "type" => 0], false).await?;
        Ok(conditions_of(&d))
    }

    async fn index(
        &self,
        season_type: u32,
        filters: &[(String, String)],
        order: &str,
        page: u32,
    ) -> Result<(Vec<PgcSeason>, bool)> {
        let mut p = params![
            "season_type" => season_type, "st" => season_type, "type" => 1, "order" => order, "sort" => 0,
            "page" => page.max(1), "pagesize" => INDEX_PAGE,
        ];
        p.extend(filters.iter().filter(|(_, v)| v != "-1").cloned());
        let d = self.http.get_json(INDEX, p, false).await?;
        let list = d["list"].as_array().map(|l| l.iter().map(season_of).collect()).unwrap_or_default();
        Ok((list, u64_of(&d["has_next"]) == 1))
    }

    async fn season(&self, season_id: u64) -> Result<PgcSeasonDetail> {
        let d = self.http.get_json(SEASON, params!["season_id" => season_id], false).await?;
        Ok(detail_of(&d))
    }

    async fn play(&self, ep_id: u64, qn: u32) -> Result<PgcPlay> {
        let d = self
            .http
            .get_json(
                PLAYURL,
                params!["ep_id" => ep_id, "qn" => qn, "fnval" => 4048, "fnver" => 0, "fourk" => 1, "from_client" => "BROWSER"],
                false,
            )
            .await?;
        Ok(play_of(&d))
    }

    fn stream_headers(&self) -> Vec<(String, String)> {
        self.http.stream_headers()
    }
}

pub(crate) fn days_of(d: &Value) -> Vec<PgcDay> {
    d.as_array()
        .map(|days| {
            days.iter()
                .map(|day| PgcDay {
                    date: str_of(&day["date"]),
                    today: u64_of(&day["is_today"]) == 1,
                    releases: day["episodes"]
                        .as_array()
                        .map(|l| {
                            l.iter()
                                .map(|e| PgcRelease {
                                    season_id: u64_of(&e["season_id"]),
                                    title: str_of(&e["title"]),
                                    cover: image_of(&e["square_cover"]),
                                    index: str_of(&e["pub_index"]),
                                    time: str_of(&e["pub_time"]),
                                    published: u64_of(&e["published"]) == 1,
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn conditions_of(d: &Value) -> PgcConditions {
    let pairs = |l: &Value, k: &str, n: &str| -> Vec<(String, String)> {
        l.as_array().map(|l| l.iter().map(|v| (str_of(&v[k]), str_of(&v[n]))).collect()).unwrap_or_default()
    };
    PgcConditions {
        filters: d["filter"]
            .as_array()
            .map(|l| {
                l.iter()
                    .map(|f| PgcFilter {
                        field: str_of(&f["field"]),
                        name: str_of(&f["name"]),
                        values: pairs(&f["values"], "keyword", "name"),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        orders: pairs(&d["order"], "field", "name"),
    }
}

pub(crate) fn season_of(v: &Value) -> PgcSeason {
    PgcSeason {
        season_id: u64_of(&v["season_id"]),
        title: str_of(&v["title"]),
        subtitle: str_of(&v["subTitle"]),
        cover: image_of(&v["cover"]),
        score: str_of(&v["score"]),
        badge: str_of(&v["badge"]),
        index_show: str_of(&v["index_show"]),
        order_text: str_of(&v["order"]),
    }
}

pub(crate) fn detail_of(d: &Value) -> PgcSeasonDetail {
    let names = |l: &Value| -> Vec<String> {
        l.as_array()
            .map(|l| l.iter().map(|v| v["name"].as_str().map(str::to_owned).unwrap_or_else(|| str_of(v))).collect())
            .unwrap_or_default()
    };
    let score = d["rating"]["score"].as_f64().map(|s| format!("{s:.1}")).unwrap_or_default();
    let pub_time = str_of(&d["publish"]["pub_time_show"]);
    PgcSeasonDetail {
        season_id: u64_of(&d["season_id"]),
        title: str_of(&d["title"]),
        cover: image_of(&d["cover"]),
        evaluate: str_of(&d["evaluate"]),
        score,
        areas: names(&d["areas"]),
        styles: names(&d["styles"]),
        pub_time: pub_time.split(' ').next().unwrap_or_default().chars().take(11).collect(),
        episodes: d["episodes"]
            .as_array()
            .map(|l| {
                l.iter()
                    .map(|e| PgcEpisode {
                        ep_id: u64_of(&e["id"]).max(u64_of(&e["ep_id"])),
                        aid: u64_of(&e["aid"]),
                        bvid: str_of(&e["bvid"]),
                        cid: u64_of(&e["cid"]),
                        title: str_of(&e["title"]),
                        long_title: str_of(&e["long_title"]),
                        badge: str_of(&e["badge"]),
                        duration_ms: u64_of(&e["duration"]),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        allow_download: u64_of(&d["rights"]["allow_download"]) == 1,
    }
}

pub(crate) fn play_of(d: &Value) -> PgcPlay {
    let vi = &d["video_info"];
    let preview = u64_of(&vi["is_preview"]) == 1 || str_of(&d["play_check"]["play_detail"]).contains("PREVIEW");
    let drm = vi["is_drm"].as_bool().unwrap_or(false) || u64_of(&vi["is_drm"]) == 1;
    PgcPlay { url: play_url_of(vi).ok(), preview, drm }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// 2026-11-03: shapes of the timeline, index, season and playurl answers.
    #[test]
    fn reads_timeline_index_and_season() {
        let days = days_of(&json!([{"date": "11-3", "is_today": 1, "episodes": [
            {"season_id": 332584, "title": "功夫熊猫", "square_cover": "https://i0.hdslb.com/s.jpg", "pub_index": "第11话", "pub_time": "10:00", "published": 1}]}]));
        assert_eq!((days[0].today, days[0].releases[0].index.as_str()), (true, "第11话"));

        let c = conditions_of(
            &json!({"filter": [{"field": "area", "name": "地区", "values": [{"keyword": "-1", "name": "全部地区"}, {"keyword": "2", "name": "日本"}]}],
            "order": [{"field": "8", "name": "综合排序"}, {"field": "4", "name": "最高评分"}]}),
        );
        assert_eq!(c.filters[0].values[1], ("2".to_string(), "日本".to_string()));
        assert_eq!(c.orders[1].1, "最高评分");

        let s = season_of(
            &json!({"badge": "大会员", "cover": "http://i0.hdslb.com/c.jpg", "index_show": "全11话", "order": "553.3万追番",
            "score": "9.6", "season_id": 835, "subTitle": "难忘的催泪夏天", "title": "未闻花名"}),
        );
        assert_eq!(
            (s.badge.as_str(), s.score.as_str(), s.cover.as_str()),
            ("大会员", "9.6", "https://i0.hdslb.com/c.jpg")
        );

        let d = detail_of(
            &json!({"season_id": 835, "title": "未闻花名", "rating": {"score": 9.6}, "areas": [{"name": "日本"}],
            "styles": ["催泪", "原创"], "publish": {"pub_time_show": "2011年04月14日00:00"}, "rights": {"allow_download": 1},
            "episodes": [{"id": 15014, "aid": 1358913, "bvid": "BV1rx411N7wm", "cid": 211874678, "title": "1", "long_title": "超和平busters", "duration": 1476000}]}),
        );
        assert_eq!((d.score.as_str(), d.pub_time.as_str(), d.allow_download), ("9.6", "2011年04月14日", true));
        assert_eq!((d.areas[0].as_str(), d.styles[1].as_str(), d.episodes[0].ep_id), ("日本", "原创", 15014));
    }

    #[test]
    fn reads_rights_of_a_play() {
        let whole = play_of(
            &json!({"play_check": {"play_detail": "PLAY_WHOLE"}, "video_info": {"is_preview": 0, "is_drm": false,
            "quality": 80, "timelength": 1000, "accept_quality": [80, 64], "support_formats": [{"quality": 80, "new_description": "1080P 高清"}, {"quality": 64, "new_description": "720P 准高清"}],
            "dash": {"video": [], "audio": []}}}),
        );
        assert!(!whole.preview && !whole.drm);
        assert_eq!(whole.url.unwrap().accept[0], (80, "1080P 高清".to_string()));
        let trial = play_of(&json!({"play_check": {"play_detail": "PLAY_PREVIEW"}, "video_info": {"is_preview": 1}}));
        assert!(trial.preview && trial.url.is_none());
        assert!(play_of(&json!({"video_info": {"is_drm": true}})).drm);
    }
}
