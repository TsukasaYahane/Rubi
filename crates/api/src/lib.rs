//! Bilibili web API client. Each domain has its own trait (`AuthApi`, `FeedApi`,
//! `VideoApi`, `DanmakuApi`) and a small implementation over a shared [`HttpClient`].
//!
//! Endpoint notes and their verification status live in `api-notes/`.

mod auth;
mod danmaku;
mod error;
mod feed;
mod http;
mod limiter;
mod parse;
mod proto;
mod session;
mod video;
pub mod wbi;

pub use auth::{AuthApi, NavInfo, QrState, QrTicket, WebAuth};
pub use danmaku::{DanmakuApi, DanmakuView, SEGMENT_MS, WebDanmaku};
pub use error::{ApiError, Result};
pub use feed::{FeedApi, WebFeed};
pub use http::HttpClient;
pub use limiter::RateLimiter;
pub use session::Session;
pub use video::{Page, PlayUrl, Stream, VideoApi, VideoDetail, WebVideo};
