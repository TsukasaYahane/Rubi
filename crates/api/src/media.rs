//! Media bytes for offline copies: a stream read chunk by chunk, from any byte on.

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE, RANGE};

use crate::error::{ApiError, Result};
use crate::http::HttpClient;

/// An open stream body.
#[async_trait]
pub trait MediaBody: Send {
    /// The whole stream's size, when the server says it.
    fn total(&self) -> Option<u64>;
    /// Whether the body starts at the byte asked for (false: the server sent it all again).
    fn resumed(&self) -> bool;
    /// The next chunk; `None` at the end.
    async fn chunk(&mut self) -> Result<Option<Vec<u8>>>;
}

#[async_trait]
pub trait MediaFetch: Send + Sync {
    /// Opens `url` from byte `from` on. Sends the stream headers players send, never cookies.
    async fn open(&self, url: &str, from: u64) -> Result<Box<dyn MediaBody>>;
}

pub struct WebMedia {
    http: Arc<HttpClient>,
}

impl WebMedia {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

struct WebBody {
    resp: reqwest::Response,
    total: Option<u64>,
    resumed: bool,
}

#[async_trait]
impl MediaBody for WebBody {
    fn total(&self) -> Option<u64> {
        self.total
    }

    fn resumed(&self) -> bool {
        self.resumed
    }

    async fn chunk(&mut self) -> Result<Option<Vec<u8>>> {
        Ok(self.resp.chunk().await?.map(|b| b.to_vec()))
    }
}

/// The total in `Content-Range: bytes a-b/total`.
fn range_total(header: &str) -> Option<u64> {
    header.rsplit('/').next()?.trim().parse().ok()
}

#[async_trait]
impl MediaFetch for WebMedia {
    async fn open(&self, url: &str, from: u64) -> Result<Box<dyn MediaBody>> {
        let range = (from > 0).then(|| format!("bytes={from}-"));
        let resp = self.http.media(url, range.as_deref().map(|r| (RANGE, r))).await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(ApiError::Decode(format!("media request answered {status}")));
        }
        let header = |name| resp.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_owned);
        let resumed = status == StatusCode::PARTIAL_CONTENT;
        let total = if resumed {
            header(CONTENT_RANGE).as_deref().and_then(range_total)
        } else {
            header(CONTENT_LENGTH).and_then(|l| l.parse().ok())
        };
        Ok(Box::new(WebBody { resp, total, resumed }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_total_of_a_range() {
        assert_eq!(range_total("bytes 100-999/1000"), Some(1000));
        assert_eq!(range_total("bytes */*"), None);
    }
}
