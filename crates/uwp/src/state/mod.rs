//! What the pages decide without touching XAML.

pub mod access;
pub mod comments;
pub mod danmaku;
pub mod emotes;
/// Where exports go: it touches XAML's storage types, so it is only built for the app.
#[cfg(target_vendor = "uwp")]
pub mod exports;
pub mod feed;
pub mod follow;
pub mod handover;
pub mod images;
pub mod layout;
pub mod library;
pub mod live;
pub mod manifest;
pub mod panes;
pub mod pgc;
pub mod pictures;
pub mod qr;
pub mod quality;
pub mod queue;
pub mod search;
pub mod settings;
pub mod watch;
