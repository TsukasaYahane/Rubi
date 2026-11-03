//! Settings, and moving them (with the profile, history and play list) between devices.

use std::fs::File;
use std::sync::Arc;

use crate::error::Result;
use crate::export::{ExportOptions, export_bundle, import_bundle};
use crate::ports::{PlaylistBackup, ProfileBackup, SettingsPort};
use crate::settings::Settings;

pub struct Preferences {
    settings: Arc<dyn SettingsPort>,
    profile: Arc<dyn ProfileBackup>,
    playlists: Arc<dyn PlaylistBackup>,
}

impl Preferences {
    pub(crate) fn new(
        settings: Arc<dyn SettingsPort>,
        profile: Arc<dyn ProfileBackup>,
        playlists: Arc<dyn PlaylistBackup>,
    ) -> Self {
        Self { settings, profile, playlists }
    }

    /// The whole settings object (rules, weights, intervals, display and playback).
    pub fn settings(&self) -> Settings {
        self.settings.settings()
    }

    /// Saves settings; invalid rules are rejected and nothing changes.
    pub fn set_settings(&self, settings: Settings) -> Result<()> {
        self.settings.set_settings(settings)
    }

    /// Changes only the keys in `patch` (a JSON merge patch), leaving the rest as they are now.
    pub fn patch_settings(&self, patch: &str) -> Result<()> {
        self.settings.patch_json(patch)
    }

    /// Writes a zip with settings, rules, weights, profile and play list (never credentials).
    pub fn export_to(&self, path: &str, include_history: bool) -> Result<()> {
        let history = if include_history { self.profile.history()? } else { Vec::new() };
        let playlists = self.playlists.export_lists()?;
        let file = File::create(path)?;
        export_bundle(
            file,
            &self.settings.settings(),
            &self.profile.profile(),
            &history,
            &playlists,
            ExportOptions { include_history },
        )
    }

    /// Replaces settings, profile and history with the bundle's, and takes its play lists when
    /// it has some. The whole bundle is read and its rules checked before anything changes;
    /// each part is then written in one go.
    pub fn import_from(&self, path: &str) -> Result<()> {
        let imported = import_bundle(File::open(path)?)?;
        imported.settings.rules.card_pipeline(None, rubi_model::now_secs)?;
        imported.settings.rules.danmaku_options()?;
        if !imported.playlists.is_empty() {
            self.playlists.replace_all(&imported.playlists)?;
        }
        self.profile.restore(imported.profile, &imported.history)?;
        self.settings.set_settings(imported.settings)
    }
}
