use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{SourceKind, VideoCard};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{card_of, duration_of, str_of, u64_of};

const VIEW: &str = "https://api.bilibili.com/x/web-interface/view";
const TAGS: &str = "https://api.bilibili.com/x/web-interface/view/detail/tag";
const PLAYURL: &str = "https://api.bilibili.com/x/player/wbi/playurl";
/// DASH + HDR + 4K + Dolby + 8K + AV1. The server still caps quality to what the account may play.
const FNVAL: u32 = 16 | 64 | 128 | 256 | 512 | 1024 | 2048;

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub cid: u64,
    pub part: String,
    pub duration_s: u32,
}

#[derive(Debug, Clone)]
pub struct VideoDetail {
    pub card: VideoCard,
    pub cid: u64,
    pub pages: Vec<Page>,
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
    async fn tags(&self, bvid: &str) -> Result<Vec<String>>;
    async fn playurl(&self, bvid: &str, cid: u64, qn: u32) -> Result<PlayUrl>;
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
        Ok(VideoDetail { cid: u64_of(&d["cid"]), card: card_of(&d, SourceKind::Related), pages })
    }

    async fn bvid_of(&self, aid: u64) -> Result<String> {
        let d = self.http.get_json(VIEW, params!["aid" => aid], false).await?;
        let bvid = str_of(&d["bvid"]);
        if bvid.is_empty() { Err(ApiError::Decode(format!("av{aid} has no bvid"))) } else { Ok(bvid) }
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
        let dash = &d["dash"];
        if dash.is_null() {
            return Err(ApiError::Decode("playurl returned no DASH streams".into()));
        }
        let mut audio = streams(&dash["audio"]);
        if dash["flac"]["audio"].is_object() {
            audio.push(stream_of(&dash["flac"]["audio"]));
        }
        audio.extend(streams(&dash["dolby"]["audio"]));
        let accept = d["accept_quality"]
            .as_array()
            .zip(d["accept_description"].as_array())
            .map(|(q, desc)| q.iter().zip(desc).map(|(q, d)| (u64_of(q) as u32, str_of(d))).collect())
            .unwrap_or_default();
        Ok(PlayUrl {
            duration_ms: u64_of(&d["timelength"]),
            quality: u64_of(&d["quality"]) as u32,
            accept,
            video: streams(&dash["video"]),
            audio,
        })
    }

    fn stream_headers(&self) -> Vec<(String, String)> {
        self.http.stream_headers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
