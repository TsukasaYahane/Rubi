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

fn stream_of(v: &Value) -> Stream {
    let url = [&v["baseUrl"], &v["base_url"]].into_iter().map(str_of).find(|s| !s.is_empty()).unwrap_or_default();
    let backup = [&v["backupUrl"], &v["backup_url"]]
        .into_iter()
        .find_map(Value::as_array)
        .map(|a| a.iter().map(str_of).collect())
        .unwrap_or_default();
    Stream {
        id: u64_of(&v["id"]) as u32,
        url,
        backup_urls: backup,
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
