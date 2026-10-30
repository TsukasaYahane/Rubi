use serde::{Deserialize, Serialize};

use crate::VideoCard;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FavFolder {
    pub id: u64,
    pub title: String,
    pub count: u32,
    /// Whether the video asked about is already in this folder.
    pub contains: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub card: VideoCard,
    /// Seconds watched; -1 means finished.
    pub progress_s: i64,
    /// Unix seconds.
    pub viewed_at: i64,
}

/// What the current user has done to a video.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoRelation {
    pub liked: bool,
    pub coins: u32,
    pub favorited: bool,
    pub following_up: bool,
}
