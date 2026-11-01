//! Domain types shared by every Rubi crate. No I/O lives here.

mod article;
mod comment;
mod danmaku;
mod download;
mod dynamic;
mod feedback;
mod library;
mod message;
mod space;
mod video;
mod video_ref;

pub use article::{Article, ArticleBlock};
pub use comment::{Comment, CommentPicture, ThreadRow, VideoLink, flatten_thread};
pub use danmaku::{CommandDanmaku, Danmaku, DanmakuMode, JumpMark};
pub use download::{Download, DownloadState};
pub use dynamic::{Dynamic, DynamicKind, FollowedUp, Following, FollowingUp};
pub use feedback::{DislikeReason, FeedbackEvent, ItemRef, Signal};
pub use library::{FavFolder, HistoryEntry, VideoRelation};
pub use message::{ChatMessage, ChatSession, CommentRef, Notice, NoticeKind, SharedVideo, UnreadCounts};
pub use space::UpProfile;
pub use video::{Owner, SourceKind, Stats, VideoCard, Zone};
pub use video_ref::{VideoRef, find_video_ref};

/// Current Unix time in seconds.
pub fn now_secs() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or_default()
}
