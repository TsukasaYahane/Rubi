use std::sync::Arc;

use async_trait::async_trait;
use prost::Message;
use rubi_model::{CommandDanmaku, Danmaku};

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::proto::{DmSegMobileReply, DmWebViewReply};

const SEG: &str = "https://api.bilibili.com/x/v2/dm/wbi/web/seg.so";
const VIEW: &str = "https://api.bilibili.com/x/v2/dm/web/view";
const POST: &str = "https://api.bilibili.com/x/v2/dm/post";
/// Each segment covers six minutes.
pub const SEGMENT_MS: u32 = 6 * 60 * 1000;

#[derive(Debug, Clone, Default)]
pub struct DanmakuView {
    pub segment_count: u32,
    pub commands: Vec<CommandDanmaku>,
}

#[async_trait]
pub trait DanmakuApi: Send + Sync {
    /// Danmaku of one 6-minute segment; `index` starts at 1.
    async fn segment(&self, cid: u64, index: u32) -> Result<Vec<Danmaku>>;
    async fn view(&self, cid: u64) -> Result<DanmakuView>;
    /// Posts a danmaku at `progress_ms`; returns its id. Login required.
    async fn send(&self, req: &SendDanmaku) -> Result<u64>;
}

/// A danmaku to post. `mode`: 1 scroll, 4 bottom, 5 top; `color` is 0xRRGGBB.
#[derive(Debug, Clone)]
pub struct SendDanmaku {
    pub bvid: String,
    pub cid: u64,
    pub progress_ms: u32,
    pub text: String,
    pub mode: u8,
    pub color: u32,
}

pub struct WebDanmaku {
    http: Arc<HttpClient>,
}

impl WebDanmaku {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl DanmakuApi for WebDanmaku {
    async fn segment(&self, cid: u64, index: u32) -> Result<Vec<Danmaku>> {
        let bytes =
            self.http.get_bytes(SEG, params!["type" => 1, "oid" => cid, "segment_index" => index], true).await?;
        let reply = DmSegMobileReply::decode(bytes.as_slice())?;
        Ok(reply
            .elems
            .into_iter()
            .map(|e| Danmaku {
                id: e.id as u64,
                progress_ms: e.progress.max(0) as u32,
                mode: e.mode.clamp(0, 255) as u8,
                font_size: e.fontsize.max(0) as u32,
                color: e.color,
                mid_hash: e.mid_hash,
                content: e.content,
                ctime: e.ctime,
                weight: e.weight,
                pool: e.pool,
                attr: e.attr,
            })
            .collect())
    }

    async fn send(&self, req: &SendDanmaku) -> Result<u64> {
        let form = params![
            "type" => 1,
            "oid" => req.cid,
            "bvid" => &req.bvid,
            "msg" => &req.text,
            "progress" => req.progress_ms,
            "mode" => req.mode,
            "color" => req.color,
            "fontsize" => 25,
            "pool" => 0,
            "plat" => 1,
            "rnd" => rubi_model::now_secs(),
        ];
        let d = self.http.post_form(POST, form).await?;
        Ok(d["dmid"].as_u64().or_else(|| d["dmid_str"].as_str().and_then(|s| s.parse().ok())).unwrap_or(0))
    }

    async fn view(&self, cid: u64) -> Result<DanmakuView> {
        let bytes = self.http.get_bytes(VIEW, params!["type" => 1, "oid" => cid], false).await?;
        let reply = DmWebViewReply::decode(bytes.as_slice())?;
        Ok(DanmakuView {
            segment_count: reply.dm_sge.map_or(0, |s| s.total.max(0) as u32),
            commands: reply
                .command_dms
                .into_iter()
                .map(|c| CommandDanmaku {
                    id: c.id as u64,
                    command: c.command,
                    content: c.content,
                    progress_ms: c.progress.max(0) as u32,
                    extra: c.extra,
                })
                .collect(),
        })
    }
}
