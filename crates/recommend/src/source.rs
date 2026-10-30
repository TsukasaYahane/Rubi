use async_trait::async_trait;
use rubi_model::{SourceKind, VideoCard};

#[derive(Debug, thiserror::Error)]
#[error("{kind:?} source failed: {message}")]
pub struct SourceError {
    pub kind: SourceKind,
    pub message: String,
}

/// A pool of candidate videos (B 站推荐, 关注 UP, 热门…).
#[async_trait]
pub trait Source: Send + Sync {
    fn kind(&self) -> SourceKind;

    /// Fetches roughly `want` new candidates. Cards are returned unfiltered.
    async fn fetch(&self, want: usize) -> Result<Vec<VideoCard>, SourceError>;
}
