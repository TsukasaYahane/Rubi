use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{image_of, str_of};

const PACKAGE: &str = "https://api.bilibili.com/x/emote/package";
/// 小黄脸, the default set used across comments, messages and danmaku. Readable as a guest.
pub const DEFAULT_PACKAGES: &[u32] = &[1];

/// Emote token (e.g. `[doge]`) and its image url.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emote {
    pub text: String,
    pub url: String,
}

#[async_trait]
pub trait EmoteApi: Send + Sync {
    async fn packages(&self, ids: &[u32]) -> Result<Vec<Emote>>;
}

pub struct WebEmote {
    http: Arc<HttpClient>,
}

impl WebEmote {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl EmoteApi for WebEmote {
    async fn packages(&self, ids: &[u32]) -> Result<Vec<Emote>> {
        let ids = ids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        let d = self.http.get_json(PACKAGE, params!["ids" => ids, "business" => "reply"], false).await?;
        Ok(d["packages"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|p| p["emote"].as_array().cloned().unwrap_or_default())
            .map(|e| Emote { text: str_of(&e["text"]), url: image_of(&e["url"]) })
            .filter(|e| e.text.starts_with('[') && !e.url.is_empty())
            .collect())
    }
}
