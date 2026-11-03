//! The app-facing API (Kotlin today, Swift later). One [`RubiCore`] object holds the core's
//! services; each module below exports one area's calls and only converts between the flat
//! records in [`types`] and the core's types. No rules or state live here.

mod account;
mod actions;
mod comments;
mod discover;
mod downloads;
mod error;
mod following;
mod home;
mod library;
mod live;
mod messages;
mod pgc;
mod player;
mod playlist;
mod reading;
mod seasons;
mod settings;
mod space;
mod types;

use std::path::PathBuf;
use std::sync::Arc;

use rubi_api::Session;
use rubi_core::{
    Account, Actions, Comments, Danmakus, Discover, Downloads, Emotes, Esports, Following, Home, Library, Live,
    Messages, Pgc, Playlists, Preferences, Reading, Seasons, Services, Space, Stein, Videos,
};

pub use error::RubiError;
pub use live::FfiLiveChannel;
pub use player::{FfiDanmakuSession, FfiStein};
pub use types::*;

uniffi::setup_scaffolding!();

type Res<T> = Result<T, RubiError>;

#[derive(uniffi::Object)]
pub struct RubiCore {
    account: Arc<Account>,
    home: Home,
    videos: Arc<Videos>,
    danmaku: Danmakus,
    stein: Stein,
    comments: Comments,
    discover: Discover,
    emotes: Emotes,
    following: Following,
    space: Space,
    actions: Actions,
    library: Library,
    messages: Messages,
    playlists: Playlists,
    preferences: Preferences,
    reading: Reading,
    downloads: Downloads,
    live: Live,
    esports: Esports,
    pgc: Pgc,
    seasons: Seasons,
}

#[uniffi::export]
impl RubiCore {
    /// `data_dir` is the app's private files directory; `session_json` is what a previous
    /// login returned (decrypted by the caller), or `None` for a guest.
    #[uniffi::constructor]
    pub fn new(data_dir: String, session_json: Option<String>) -> Res<Arc<Self>> {
        let session: Session = session_json.as_deref().map(serde_json::from_str).transpose()?.unwrap_or_default();
        let Services {
            account,
            home,
            videos,
            danmaku,
            stein,
            comments,
            discover,
            following,
            space,
            actions,
            library,
            messages,
            playlists,
            preferences,
            emotes,
            reading,
            downloads,
            live,
            esports,
            pgc,
            seasons,
        } = Services::open(PathBuf::from(data_dir), session)?;
        Ok(Arc::new(Self {
            account,
            home,
            videos,
            danmaku,
            stein,
            comments,
            discover,
            emotes,
            following,
            space,
            actions,
            library,
            messages,
            playlists,
            preferences,
            reading,
            downloads,
            live,
            esports,
            pgc,
            seasons,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_core_opens_and_ranks_an_empty_pool() {
        let dir = std::env::temp_dir().join(format!("rubi-ffi-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let core = RubiCore::new(dir.to_string_lossy().into_owned(), None).unwrap();
        assert!(core.next_page(10).unwrap().is_empty());
        assert!(core.settings_json().unwrap().contains("rules"));
        drop(core);
        let _ = std::fs::remove_dir_all(dir);
    }
}
