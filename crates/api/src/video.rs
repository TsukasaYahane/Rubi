use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{SourceKind, VideoCard};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{card_of, duration_of, image_of, plain_letters, str_of, u64_of};

const VIEW: &str = "https://api.bilibili.com/x/web-interface/view";
const TAGS: &str = "https://api.bilibili.com/x/web-interface/view/detail/tag";
const PLAYURL: &str = "https://api.bilibili.com/x/player/wbi/playurl";
/// Player info; for a signed-in user it also says where they stopped watching.
const PLAYER: &str = "https://api.bilibili.com/x/player/v2";
/// DASH + HDR + 4K + Dolby + 8K + AV1. The server still caps quality to what the account may play.
const FNVAL: u32 = 16 | 64 | 128 | 256 | 512 | 1024 | 2048;

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub cid: u64,
    pub part: String,
    pub duration_s: u32,
}

/// One video of a 合集 (UGC season).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeasonEpisode {
    pub bvid: String,
    pub aid: u64,
    pub cid: u64,
    pub title: String,
    pub cover: String,
    pub duration_s: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeasonSection {
    pub title: String,
    pub episodes: Vec<SeasonEpisode>,
}

/// The 合集 a video belongs to, in the UP's order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Season {
    pub id: u64,
    pub title: String,
    pub sections: Vec<SeasonSection>,
}

#[derive(Debug, Clone)]
pub struct VideoDetail {
    pub card: VideoCard,
    pub cid: u64,
    pub pages: Vec<Page>,
    /// `None` when the video is in no 合集.
    pub season: Option<Season>,
}

/// `last_play_time` (ms) and `last_play_cid` of `player/v2`; a missing cid means the part asked about.
fn last_watched_of(d: &Value, cid: u64) -> Option<(u64, u64)> {
    let at = u64_of(&d["last_play_time"]);
    let part = Some(u64_of(&d["last_play_cid"])).filter(|&c| c > 0).unwrap_or(cid);
    (at > 0).then_some((part, at))
}

/// `ugc_season` of `view`: sections of episodes; a season without episodes counts as none.
fn season_of(v: &Value) -> Option<Season> {
    let sections: Vec<SeasonSection> = v["sections"]
        .as_array()?
        .iter()
        .map(|s| SeasonSection {
            title: str_of(&s["title"]),
            episodes: s["episodes"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|e| SeasonEpisode {
                            bvid: str_of(&e["bvid"]),
                            aid: u64_of(&e["aid"]),
                            cid: u64_of(&e["cid"]),
                            title: plain_letters(&str_of(&e["title"])),
                            cover: image_of(&e["arc"]["pic"]),
                            duration_s: [&e["arc"]["duration"], &e["page"]["duration"]]
                                .into_iter()
                                .map(duration_of)
                                .find(|&d| d > 0)
                                .unwrap_or_default(),
                        })
                        .filter(|e| !e.bvid.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
        })
        .filter(|s| !s.episodes.is_empty())
        .collect();
    (!sections.is_empty()).then(|| Season { id: u64_of(&v["id"]), title: str_of(&v["title"]), sections })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub id: u32,
    pub url: String,
    pub backup_urls: Vec<String>,
    pub bandwidth: u64,
    pub codecs: String,
    pub width: u32,
    pub height: u32,
}

/// A whole video as one file, with the qualities it can come in.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SingleFile {
    pub url: String,
    pub backup_urls: Vec<String>,
    pub quality: u32,
    pub accept: Vec<(u32, String)>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayUrl {
    pub duration_ms: u64,
    pub quality: u32,
    /// (qn, description) the account may choose from.
    pub accept: Vec<(u32, String)>,
    pub video: Vec<Stream>,
    /// Audio streams; also what an audio-only mode would play.
    pub audio: Vec<Stream>,
}

#[async_trait]
pub trait VideoApi: Send + Sync {
    async fn view(&self, bvid: &str) -> Result<VideoDetail>;
    /// Looks up the BV id of an av number (old links and share cards still use them).
    async fn bvid_of(&self, aid: u64) -> Result<String>;
    /// Where a share short link points (`None` when it does not redirect).
    async fn expand_short(&self, url: &str) -> Result<Option<String>>;
    async fn tags(&self, bvid: &str) -> Result<Vec<String>>;
    async fn playurl(&self, bvid: &str, cid: u64, qn: u32) -> Result<PlayUrl>;
    /// One MP4 file (video and sound together) for 投屏: TVs cannot join DASH streams.
    async fn single_file(&self, bvid: &str, cid: u64, qn: u32) -> Result<SingleFile>;
    /// Where the signed-in user stopped watching: the part (`cid`) and the position in ms.
    /// `None` for guests and for videos never watched.
    async fn last_watched(&self, bvid: &str, cid: u64) -> Result<Option<(u64, u64)>>;
    /// Headers the player must send with stream requests.
    fn stream_headers(&self) -> Vec<(String, String)>;
}

pub struct WebVideo {
    http: Arc<HttpClient>,
}

impl WebVideo {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

/// Lower is better. Official upos nodes start fastest; PCDN / mcdn nodes (peer caches on
/// odd ports or third-party hosts) often stall on the first bytes.
fn cdn_rank(url: &str) -> u8 {
    let host = url.split("://").nth(1).and_then(|r| r.split('/').next()).unwrap_or_default();
    let peer = host.contains("mcdn.")
        || host.contains("szbdyd.com")
        || host.contains(':')
        || host.split('.').all(|p| p.chars().all(|c| c.is_ascii_digit()));
    match () {
        _ if peer => 3,
        _ if host.starts_with("upos-") && host.ends_with(".bilivideo.com") => 0,
        _ if host.ends_with(".bilivideo.com") || host.ends_with(".bilivideo.cn") => 1,
        _ => 2,
    }
}

fn stream_of(v: &Value) -> Stream {
    let base = [&v["baseUrl"], &v["base_url"]].into_iter().map(str_of).find(|s| !s.is_empty());
    let backups = [&v["backupUrl"], &v["backup_url"]]
        .into_iter()
        .find_map(Value::as_array)
        .map(|a| a.iter().map(str_of).filter(|s| !s.is_empty()).collect::<Vec<_>>())
        .unwrap_or_default();
    // Stable sort: equally ranked urls keep the server's order.
    let mut urls: Vec<String> = base.into_iter().chain(backups).collect();
    urls.sort_by_key(|u| cdn_rank(u));
    let mut urls = urls.into_iter();
    Stream {
        id: u64_of(&v["id"]) as u32,
        url: urls.next().unwrap_or_default(),
        backup_urls: urls.collect(),
        bandwidth: u64_of(&v["bandwidth"]),
        codecs: str_of(&v["codecs"]),
        width: u64_of(&v["width"]) as u32,
        height: u64_of(&v["height"]) as u32,
    }
}

/// A playurl answer (videos' `data`, or a 番剧 episode's `video_info`) as DASH streams. Quality
/// names come from `accept_description`, or from `support_formats` where that is missing.
pub(crate) fn play_url_of(d: &Value) -> Result<PlayUrl> {
    let dash = &d["dash"];
    if dash.is_null() {
        return Err(ApiError::Decode("playurl returned no DASH streams".into()));
    }
    let mut audio = streams(&dash["audio"]);
    if dash["flac"]["audio"].is_object() {
        audio.push(stream_of(&dash["flac"]["audio"]));
    }
    audio.extend(streams(&dash["dolby"]["audio"]));
    let named: Vec<(u32, String)> = d["support_formats"]
        .as_array()
        .map(|l| l.iter().map(|f| (u64_of(&f["quality"]) as u32, str_of(&f["new_description"]))).collect())
        .unwrap_or_default();
    let accept = d["accept_quality"]
        .as_array()
        .map(|q| {
            q.iter()
                .enumerate()
                .map(|(i, q)| {
                    let qn = u64_of(q) as u32;
                    let name = d["accept_description"][i]
                        .as_str()
                        .map(str::to_owned)
                        .or_else(|| named.iter().find(|(n, _)| *n == qn).map(|(_, s)| s.clone()))
                        .unwrap_or_else(|| qn.to_string());
                    (qn, name)
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(PlayUrl {
        duration_ms: u64_of(&d["timelength"]),
        quality: u64_of(&d["quality"]) as u32,
        accept,
        video: streams(&dash["video"]),
        audio,
    })
}

/// The `durl` answer (`fnval` 1): one file when the video comes whole.
pub(crate) fn single_file_of(d: &Value) -> Option<SingleFile> {
    let first = d["durl"].as_array()?.first()?;
    let url = str_of(&first["url"]);
    if url.is_empty() {
        return None;
    }
    let accept = d["accept_quality"]
        .as_array()
        .zip(d["accept_description"].as_array())
        .map(|(q, desc)| q.iter().zip(desc).map(|(q, d)| (u64_of(q) as u32, str_of(d))).collect())
        .unwrap_or_default();
    Some(SingleFile {
        url,
        backup_urls: first["backup_url"].as_array().map(|l| l.iter().map(str_of).collect()).unwrap_or_default(),
        quality: u64_of(&d["quality"]) as u32,
        accept,
        duration_ms: u64_of(&d["timelength"]),
    })
}

fn streams(list: &Value) -> Vec<Stream> {
    list.as_array().map(|a| a.iter().map(stream_of).collect()).unwrap_or_default()
}

#[async_trait]
impl VideoApi for WebVideo {
    async fn view(&self, bvid: &str) -> Result<VideoDetail> {
        let d = self.http.get_json(VIEW, params!["bvid" => bvid], false).await?;
        let pages = d["pages"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|p| Page {
                        cid: u64_of(&p["cid"]),
                        part: str_of(&p["part"]),
                        duration_s: duration_of(&p["duration"]),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(VideoDetail {
            cid: u64_of(&d["cid"]),
            card: card_of(&d, SourceKind::Related),
            pages,
            season: season_of(&d["ugc_season"]),
        })
    }

    async fn bvid_of(&self, aid: u64) -> Result<String> {
        let d = self.http.get_json(VIEW, params!["aid" => aid], false).await?;
        let bvid = str_of(&d["bvid"]);
        if bvid.is_empty() { Err(ApiError::Decode(format!("av{aid} has no bvid"))) } else { Ok(bvid) }
    }

    async fn expand_short(&self, url: &str) -> Result<Option<String>> {
        self.http.redirect_target(url).await
    }

    async fn tags(&self, bvid: &str) -> Result<Vec<String>> {
        let d = self.http.get_json(TAGS, params!["bvid" => bvid], false).await?;
        Ok(d.as_array()
            .map(|a| a.iter().map(|t| str_of(&t["tag_name"])).filter(|t| !t.is_empty()).collect())
            .unwrap_or_default())
    }

    async fn playurl(&self, bvid: &str, cid: u64, qn: u32) -> Result<PlayUrl> {
        let p = params!["bvid" => bvid, "cid" => cid, "qn" => qn, "fnval" => FNVAL, "fnver" => 0, "fourk" => 1];
        let d = self.http.get_json(PLAYURL, p, true).await?;
        play_url_of(&d)
    }

    async fn single_file(&self, bvid: &str, cid: u64, qn: u32) -> Result<SingleFile> {
        let p = params!["bvid" => bvid, "cid" => cid, "qn" => qn, "fnval" => 1, "fnver" => 0, "platform" => "html5", "high_quality" => 1];
        let d = self.http.get_json(PLAYURL, p, true).await?;
        single_file_of(&d).ok_or_else(|| ApiError::Decode("playurl returned no single file".into()))
    }

    fn stream_headers(&self) -> Vec<(String, String)> {
        self.http.stream_headers()
    }

    async fn last_watched(&self, bvid: &str, cid: u64) -> Result<Option<(u64, u64)>> {
        let d = self.http.get_json(PLAYER, params!["bvid" => bvid, "cid" => cid], false).await?;
        Ok(last_watched_of(&d, cid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_single_file() {
        let d = serde_json::json!({"quality": 64, "timelength": 1000, "accept_quality": [80, 64], "accept_description": ["高清 1080P", "高清 720P"],
            "durl": [{"url": "https://upos/x.mp4", "backup_url": ["https://b/x.mp4"]}]});
        let f = single_file_of(&d).unwrap();
        assert_eq!((f.url.as_str(), f.quality, f.backup_urls.len(), f.accept[0].0), ("https://upos/x.mp4", 64, 1, 80));
        assert!(single_file_of(&serde_json::json!({"dash": {}})).is_none());
    }

    #[test]
    fn last_watched_needs_a_position() {
        let d = serde_json::json!({"last_play_time": 432000, "last_play_cid": 22});
        assert_eq!(last_watched_of(&d, 11), Some((22, 432000)));
        assert_eq!(last_watched_of(&serde_json::json!({"last_play_time": 5000}), 11), Some((11, 5000)));
        assert_eq!(last_watched_of(&serde_json::json!({"last_play_time": 0}), 11), None);
    }

    #[test]
    fn season_lists_sections_and_episodes() {
        let v = serde_json::json!({"id": 9, "title": "合集", "sections": [
            {"title": "正片", "episodes": [
                {"bvid": "BV1a", "aid": 1, "cid": 11, "title": "第一集", "arc": {"pic": "http://i0.hdslb.com/a.jpg", "duration": 125}},
                {"bvid": "BV1b", "aid": 2, "cid": 22, "title": "第二集", "arc": {"pic": ""}, "page": {"duration": 60}}
            ]},
            {"title": "空", "episodes": []}
        ]});
        let s = season_of(&v).unwrap();
        assert_eq!((s.id, s.sections.len()), (9, 1));
        let e = &s.sections[0].episodes;
        assert_eq!((e[0].cover.as_str(), e[0].duration_s), ("https://i0.hdslb.com/a.jpg", 125));
        assert_eq!(e[1].duration_s, 60);
        assert!(season_of(&serde_json::json!(null)).is_none());
    }
    use serde_json::json;

    #[test]
    fn prefers_official_cdn_over_peer_nodes() {
        let v = json!({
            "id": 32,
            "baseUrl": "https://xy1x2x3x4xy.mcdn.bilivideo.cn:4483/upgcxcode/a.m4s",
            "backupUrl": [
                "https://cn-gddg-ct-01-01.bilivideo.com/upgcxcode/a.m4s",
                "https://upos-sz-mirrorali.bilivideo.com/upgcxcode/a.m4s"
            ]
        });
        let s = stream_of(&v);
        assert_eq!(s.url, "https://upos-sz-mirrorali.bilivideo.com/upgcxcode/a.m4s");
        assert_eq!(s.backup_urls[0], "https://cn-gddg-ct-01-01.bilivideo.com/upgcxcode/a.m4s");
        assert!(s.backup_urls[1].contains("mcdn"), "peer node kept as last resort");
    }

    #[test]
    fn ranks_hosts() {
        assert_eq!(cdn_rank("https://upos-sz-estgoss.bilivideo.com/x"), 0);
        assert_eq!(cdn_rank("https://cn-hk-eq-01-13.bilivideo.com/x"), 1);
        assert_eq!(cdn_rank("https://123.45.67.89/x"), 3);
        assert_eq!(cdn_rank("https://abc.szbdyd.com:8082/x"), 3);
    }
}
