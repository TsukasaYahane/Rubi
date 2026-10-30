use serde::{Deserialize, Serialize};

/// A regular danmaku from the segmented protobuf stream.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Danmaku {
    pub id: u64,
    pub progress_ms: u32,
    pub mode: u8,
    pub font_size: u32,
    pub color: u32,
    pub mid_hash: String,
    pub content: String,
    /// Unix seconds when it was sent.
    pub ctime: i64,
    pub weight: i32,
    pub pool: i32,
    pub attr: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DanmakuMode {
    Scroll,
    Bottom,
    Top,
    Reverse,
    Advanced,
    Code,
    Bas,
    Unknown,
}

impl Danmaku {
    pub fn kind(&self) -> DanmakuMode {
        match self.mode {
            1..=3 => DanmakuMode::Scroll,
            4 => DanmakuMode::Bottom,
            5 => DanmakuMode::Top,
            6 => DanmakuMode::Reverse,
            7 => DanmakuMode::Advanced,
            8 => DanmakuMode::Code,
            9 => DanmakuMode::Bas,
            _ => DanmakuMode::Unknown,
        }
    }
}

/// An interactive "command" danmaku (#UP#, #LINK#, #ATTENTION#, votes…).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommandDanmaku {
    pub id: u64,
    pub command: String,
    pub content: String,
    pub progress_ms: u32,
    pub extra: String,
}

/// A clickable jump point on the progress bar, produced by the 空降 adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JumpMark {
    pub target_ms: u32,
    pub label: String,
    /// Id of the danmaku that was kept for this cluster.
    pub source_id: u64,
}
