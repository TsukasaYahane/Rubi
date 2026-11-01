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

    /// Writes a zip with settings, rules, weights and profile (never credentials).
    pub fn export_to(&self, path: String, include_history: bool) -> Res<()> {
        Ok(self.preferences.export_to(&path, include_history)?)
    }

    pub fn import_from(&self, path: String) -> Res<()> {
        Ok(self.preferences.import_from(&path)?)
    }
}
