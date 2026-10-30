//! Bilibili web API client. Each domain has its own trait (`AuthApi`, `FeedApi`,
//! `VideoApi`, `DanmakuApi`) and a small implementation over a shared [`HttpClient`].
//!
//! Endpoint notes and their verification status live in `api-notes/`.

mod action;
mod auth;
mod comment;
mod danmaku;
mod emote;
mod error;
mod feed;
mod http;
mod limiter;
mod message;
mod parse;
mod proto;
mod search;
mod session;
mod video;
pub mod wbi;

pub use action::{ActionApi, LibraryApi, WebAccount};
pub use auth::{AuthApi, NavInfo, QrState, QrTicket, WebAuth};
pub use comment::{CommentApi, CommentPage, CommentSort, ThreadPreview, WebComment};
pub use danmaku::{DanmakuApi, DanmakuView, SEGMENT_MS, WebDanmaku};
pub use emote::{DEFAULT_PACKAGES, Emote, EmoteApi, WebEmote};
pub use error::{ApiError, Result};
pub use feed::{FeedApi, WebFeed};
pub use http::HttpClient;
pub use limiter::RateLimiter;
pub use message::{ChatApi, NoticeApi, NoticeCursor, WebMessages};
pub use search::{HotWord, SearchApi, SearchOrder, WebSearch};
pub use session::Session;
pub use video::{Page, PlayUrl, Stream, VideoApi, VideoDetail, WebVideo};
