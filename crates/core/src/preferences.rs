//! Settings, and moving them (with the profile, history and play list) between devices.

use std::fs::File;
use std::sync::Arc;

use rubi_store::SqliteStore;

use crate::error::Result;
use crate::export::{ExportOptions, export_bundle, import_bundle};
use crate::feed::{FeedService, Settings};
use crate::playlist::Playlist;

pub struct Preferences {
    feed: Arc<FeedService<SqliteStore>>,
    playlist: Arc<Playlist<SqliteStore>>,
}

impl Preferences {
    pub(crate) fn new(feed: Arc<FeedService<SqliteStore>>, playlist: Arc<Playlist<SqliteStore>>) -> Self {
        Self { feed, playlist }
    }

    /// The whole settings object (rules, weights, intervals, display and playback).
    pub fn settings(&self) -> Settings {
        self.feed.settings()
    }

    /// Saves settings; invalid rules are rejected and nothing changes.
    pub fn set_settings(&self, settings: Settings) -> Result<()> {
        self.feed.set_settings(settings)
    }

    /// Writes a zip with settings, rules, weights, profile and play list (never credentials).
    pub fn export_to(&self, path: &str, include_history: bool) -> Result<()> {
        let history = if include_history { self.feed.history()? } else { Vec::new() };
        let file = File::create(path)?;
        let playlists = self
            .playlist
            .lists()?
            .into_iter()
            .map(|l| Ok((l.name, self.playlist.items(l.id)?)))
            .collect::<Result<Vec<_>>>()?;
        export_bundle(
            file,
            &self.feed.settings(),
            &self.feed.profile(),
            &history,
            &playlists,
            ExportOptions { include_history },
        )
    }

    /// Replaces settings and profile with the bundle's, replays its history, and takes its
    /// play list when it has one.
    pub fn import_from(&self, path: &str) -> Result<()> {
        let imported = import_bundle(File::open(path)?)?;
        self.feed.set_settings(imported.settings)?;
        self.feed.set_profile(imported.profile)?;
        for ev in imported.history {
            self.feed.record(ev)?;
        }
        if !imported.playlists.is_empty() {
            self.playlist.replace_all(&imported.playlists)?;
        }
        Ok(())
    }
}
