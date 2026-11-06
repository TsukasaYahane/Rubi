//! The signed-in session, kept in the system's credential vault and nowhere else.

use rubi_core::Session;
use windows_core::{Interface, Result};

use crate::xaml::{IPasswordCredential, IPasswordVault, PasswordCredential, PasswordVault};

const RESOURCE: &str = "Rubi";
const USER: &str = "session";

fn vault() -> Result<IPasswordVault> {
    PasswordVault::new()?.cast()
}

/// The stored session, or a guest's when there is none (the vault fails a lookup it cannot
/// answer, so any failure means the same).
pub fn load() -> Session {
    let stored = || -> Result<String> {
        let credential = vault()?.Retrieve(RESOURCE, USER)?.cast::<IPasswordCredential>()?;
        credential.RetrievePassword()?;
        credential.Password()
    };
    stored().ok().and_then(|json| serde_json::from_str(&json).ok()).unwrap_or_default()
}

/// Forgets the stored session; there being none is fine.
pub fn clear() -> Result<()> {
    let vault = vault()?;
    match vault.Retrieve(RESOURCE, USER) {
        Ok(credential) => vault.Remove(&credential),
        Err(_) => Ok(()),
    }
}

/// Replaces the stored session.
pub fn save(session: &Session) -> Result<()> {
    let Ok(json) = serde_json::to_string(session) else { return Ok(()) };
    vault()?.Add(&PasswordCredential::CreatePasswordCredential(RESOURCE, USER, &json)?)
}
