use serde::{Deserialize, Serialize};

use crate::VideoCard;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadState {
    /// Waiting for its turn or just started.
    Queued,
    Running,
    Paused,
    Done,
    /// Stopped by an error; starting it again resumes from what is on disk.
    Failed(String),
}

/// One video kept for offline playback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Download {
    pub card: VideoCard,
    /// qn of the video stream asked for.
    pub quality: u32,
    pub state: DownloadState,
    pub done_bytes: u64,
    /// Video plus audio; an estimate until both streams have answered.
    pub total_bytes: u64,
    /// Unix seconds.
    pub created_at: i64,
}
