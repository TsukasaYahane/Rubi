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
mod esports;
mod export;
mod favorites;
mod feed;
mod following;
mod history;
mod home;
mod lineup;
mod listening;
mod live;
mod messages;
mod pgc;
mod play_queue;
mod playlist;
mod portrait;
mod ports;
mod preferences;
mod queue;
mod reading;
mod recent;
mod seasons;
mod settings;
mod social;
mod sources;
mod space;
mod stein;
mod videos;
mod watch_later;
mod zones;

pub use zones::HOME_ZONES;

pub use account::{Account, Known};
pub use actions::Actions;
pub use app::Services;
pub use comments::{Comments, CommentsPage, ThreadView};
pub use danmaku::{DanmakuSession, Danmakus};
pub use discover::Discover;
pub use downloads::{Downloads, LocalCopy, pick_streams};
pub use emotes::Emotes;
pub use error::{CoreError, Result};
pub use esports::Esports;
pub use favorites::Favorites;
pub use feed::RefreshReport;
pub use following::{AllFollowings, FollowedPage, Following};
pub use history::History;
pub use home::Home;
pub use listening::Listening;
pub use live::{Live, LiveBatch, LiveChannel, LiveFilter};
pub use messages::Messages;
pub use pgc::{DRM_ONLY, NEED_RIGHTS, PGC_ZONES, Pgc};
pub use play_queue::PlayQueue;
pub use playlist::{MAX_PLAYLIST_NAME, PlaylistMode, Playlists, RELATED_PLAYLIST, TEMP_PLAYLIST};
pub use portrait::{Picked, PortraitFeed, Sorted};
pub use preferences::Preferences;
pub use reading::Reading;
pub use seasons::{Seasons, Subscription};
pub use settings::{DanmakuDisplay, FeedSettings, PlaybackSettings, PreviewSettings, Settings};
pub use watch_later::WatchLater;
// The lower crates' types the app-facing layer converts; it depends on this crate only.
/// One part (分P) of a video.
pub use rubi_api::Page as VideoPart;
pub use rubi_api::{
    ApiError, CommentArea, PgcConditions, PgcDay, PgcEpisode, PgcFilter, PgcHit, PgcRelease, PgcSeason,
    PgcSeasonDetail, QrState, Season, SeasonEpisode, SeasonSection, SegmentRanges, SendDanmaku, Session, SolvedCaptcha,
    SteinNode, Stream, Triple, UpOrder, https_image, plain_letters,
};
pub use rubi_filter::danmaku::DanmakuOutput;
pub use rubi_filter::danmaku::layout::{Lane, Placement};
pub use rubi_recommend::{FactorId, ScoreBreakdown};
pub use rubi_store::{DEFAULT_PLAYLIST, PlaylistInfo};
pub use space::{Space, UpUploads};
pub use stein::{Stein, SteinSession};
pub use videos::{MAX_FREE_QN, Playback, ResumePoint, VideoInfo, Videos, portrait_of};
