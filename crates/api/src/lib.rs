//! Bilibili web API client. Each domain has its own trait (`AuthApi`, `FeedApi`,
//! `VideoApi`, `DanmakuApi`) and a small implementation over a shared [`HttpClient`].
//!
//! Endpoint notes and their verification status live in `api-notes/`.

mod action;
mod article;
mod auth;
mod comment;
mod danmaku;
mod dynamic;
mod emote;
mod error;
mod esports;
mod feed;
mod http;
mod limiter;
mod live;
mod live_socket;
mod media;
mod message;
mod parse;
mod pgc;
mod proto;
mod search;
mod season;
mod session;
mod space;
mod stein;
mod video;
pub mod wbi;

pub use action::{FavoritesApi, FollowingsApi, HistoryApi, Triple, VideoActionApi, WatchLaterApi, WebAccount};
pub use article::{ArticleApi, WebArticle};
pub use auth::{AuthApi, LoginCaptcha, LoginKey, NavInfo, QrState, QrTicket, SolvedCaptcha, WebAuth};
pub use comment::{CommentApi, CommentArea, CommentPage, CommentSort, ThreadPreview, WebComment};
pub use danmaku::{DanmakuApi, DanmakuView, SEGMENT_MS, SendDanmaku, WebDanmaku};
pub use dynamic::{DynamicApi, WebDynamic};
pub use emote::{DEFAULT_PACKAGES, Emote, EmoteApi, WebEmote};
pub use error::{ApiError, Result};
pub use esports::{EsportsApi, MatchPage, WebEsports};
pub use feed::{FeedApi, WebFeed};
pub use http::{HttpClient, SessionHolder};
pub use limiter::RateLimiter;
pub use live::{BEST_LIVE_QN, DanmuServer, LiveApi, WebLive, event_of};
pub use live_socket::{ChannelJoin, ChannelOpener, ChannelStream, WebChannels};
pub use media::{MediaBody, MediaFetch, WebMedia};
pub use message::{ChatApi, NoticeApi, NoticeCursor, VideoShare, WebMessages};
pub use parse::{https_image, plain_letters};
pub use pgc::{
    PgcApi, PgcConditions, PgcDay, PgcEpisode, PgcFilter, PgcPlay, PgcRelease, PgcSeason, PgcSeasonDetail, WebPgc,
};
pub use search::{HotWord, PgcHit, PgcSearch, SearchApi, SearchOrder, UserHit, UserHits, WebSearch};
pub use season::{SEASON_PAGE, SeasonApi, WebSeasons};
pub use session::Session;
pub use space::{SpaceApi, UpArticle, UpArticles, UpOrder, UpVideo, UpVideos, WebSpace};
pub use stein::{SteinApi, SteinChoice, SteinNode, SteinQuestion, SteinVar, WebStein};
pub use video::{
    Page, PlayUrl, Season, SeasonEpisode, SeasonSection, SingleFile, Stream, VideoApi, VideoDetail, WebVideo,
};
