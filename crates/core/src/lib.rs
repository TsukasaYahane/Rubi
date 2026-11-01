//! Rubi's application layer. [`Services`] wires every service once; each service owns one
//! area (account, videos, comments, lists…), keeps its collaborators private, and talks to
//! others only through narrow traits. The FFI layer and the CLI only convert and call.

mod account;
mod actions;
mod app;
mod comments;
mod danmaku;
mod discover;
mod downloads;
mod emotes;
mod error;
mod export;
mod feed;
mod following;
mod home;
mod library;
mod messages;
mod playlist;
mod ports;
mod preferences;
mod queue;
mod reading;
mod recent;
mod social;
mod sources;
mod space;
mod stein;
mod videos;
mod zones;

pub use account::Account;
pub use actions::Actions;
pub use app::Services;
pub use comments::{Comments, CommentsPage, ThreadView};
pub use danmaku::{DanmakuSession, Danmakus};
pub use discover::Discover;
pub use downloads::{Downloads, LocalCopy, pick_streams};
pub use emotes::Emotes;
pub use error::{CoreError, Result};
pub use feed::{DanmakuDisplay, PreviewSettings, RefreshReport, Settings};
pub use following::{AllFollowings, FollowedPage, Following};
pub use home::Home;
pub use library::Library;
pub use messages::Messages;
pub use playlist::{PlaylistMode, Playlists};
pub use preferences::Preferences;
pub use reading::Reading;
pub use rubi_api::CommentArea;
pub use space::{Space, UpUploads};
pub use stein::{Stein, SteinSession};
pub use videos::{MAX_FREE_QN, Playback, ResumePoint, VideoInfo, Videos};
