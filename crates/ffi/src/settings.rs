//! Settings and backup.

use crate::{Res, RubiCore};

#[uniffi::export]
impl RubiCore {
    /// The whole settings object (rules, weights, intervals) as JSON.
    pub fn settings_json(&self) -> Res<String> {
        Ok(serde_json::to_string(&self.preferences.settings())?)
    }

    pub fn set_settings_json(&self, json: String) -> Res<()> {
        Ok(self.preferences.set_settings(serde_json::from_str(&json)?)?)
    }

    /// Changes only the keys in `patch` (a JSON merge patch, e.g. `{"auto_pip": true}` or
    /// `{"danmaku_display": {"opacity": 0.5}}`); everything else stays as it is now, so two
    /// pages saving different settings never undo each other.
    pub fn patch_settings_json(&self, patch: String) -> Res<()> {
        Ok(self.preferences.patch_settings(&patch)?)
    }

    /// Writes a zip with settings, rules, weights and profile (never credentials).
    pub fn export_to(&self, path: String, include_history: bool) -> Res<()> {
        Ok(self.preferences.export_to(&path, include_history)?)
    }

    pub fn import_from(&self, path: String) -> Res<()> {
        Ok(self.preferences.import_from(&path)?)
    }
}
