//! The composition root. Builds every service from its parts and is the only place
//! that knows concrete types; the FFI layer and the CLI talk to [`App`].

mod app;
mod danmaku;
mod error;
mod export;
mod feed;
mod social;
mod sources;

pub use app::App;
pub use danmaku::load_danmaku;
pub use error::{CoreError, Result};
pub use export::{ExportOptions, Imported, export_bundle, import_bundle};
pub use feed::{AppStore, FeedService, RefreshReport, Seeded, Settings};
pub use social::{FilteredComments, filter_hot_words, filter_sessions, is_blocked_pinned, load_comments, load_thread};
pub use sources::{
    BiliRecommendSource, FollowedUpsSource, PopularSource, PreferredZoneSource, RankingSource, RelatedSource,
};
