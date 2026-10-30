//! Domain types shared by every Rubi crate. No I/O lives here.

mod comment;
mod danmaku;
mod feedback;
mod video;

pub use comment::{Comment, ThreadRow, flatten_thread};
pub use danmaku::{CommandDanmaku, Danmaku, DanmakuMode, JumpMark};
pub use feedback::{DislikeReason, FeedbackEvent, ItemRef, Signal};
pub use video::{Owner, SourceKind, Stats, VideoCard, Zone};

/// Current Unix time in seconds.
pub fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or_default()
}
