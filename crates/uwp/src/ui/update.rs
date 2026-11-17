//! 检查更新: which copy this is decides where a newer one comes from, and whether there is one.

use crate::ui::post;
use crate::ui::shell::shell;
use crate::xaml::{Launcher, Uri};

windows_link::link!(
    "kernel32.dll" "system" fn GetCurrentPackageFullName(
        package_full_name_length: *mut u32,
        package_full_name: *mut u16,
    ) -> i32
);

/// Where the side-loaded builds are published.
const REPO: &str = "https://github.com/TsukasaYahane/Rubi/releases/latest";
/// The same releases, asked what the newest one is called.
const API: &str = "https://api.github.com/repos/TsukasaYahane/Rubi/releases/latest";
/// The name the store publishes this app under: a package with this name updates from there.
const STORE_NAME: &str = "75B213EC.MetroBi";
/// `GetCurrentPackageFullName`'s answer when the process was not started from a package.
const NO_PACKAGE: i32 = 15_700;

/// The full package name, `Name_Version_Arch_ResourceId_PublisherId`, or `None` when this is not
/// running from a package at all.
fn full_name() -> Option<String> {
    let mut length = 0u32;
    // Ask for the size first: the name is not a fixed length.
    let probe = unsafe { GetCurrentPackageFullName(&mut length, std::ptr::null_mut()) };
    if probe == NO_PACKAGE || length == 0 {
        return None;
    }
    let mut buffer = vec![0u16; length as usize];
    let answer = unsafe { GetCurrentPackageFullName(&mut length, buffer.as_mut_ptr()) };
    if answer != 0 {
        return None;
    }
    // The length includes the terminating nul.
    let used = length.saturating_sub(1) as usize;
    Some(String::from_utf16_lossy(&buffer[..used]))
}

fn part(at: usize) -> Option<String> {
    full_name()?.split('_').nth(at).map(str::to_owned)
}

fn package_name() -> Option<String> {
    part(0)
}

/// The version the package manager sees, which is the one an update would replace.
pub fn version() -> String {
    part(1).unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned())
}

/// 微软商店 or GitHub, for the settings page to show next to the version.
pub fn channel() -> &'static str {
    if package_name().as_deref() == Some(STORE_NAME) {
        "微软商店"
    } else {
        "GitHub"
    }
}

/// Opens wherever this copy is updated from.
pub fn go(target: &str) {
    let Ok(uri) = Uri::CreateUri(target) else { return };
    // Not waited on: the answer arrives through the ui thread's own message pump.
    let _ = Launcher::LaunchUriAsync(&uri);
}

/// `remote` is a later version than `local`, comparing the numbers of each part in turn.
fn newer(remote: &str, local: &str) -> bool {
    let numbers = |text: &str| -> Vec<u32> { text.split('.').filter_map(|part| part.parse().ok()).collect() };
    let (remote, local) = (numbers(remote), numbers(local));
    if remote.is_empty() {
        return false;
    }
    for at in 0..remote.len().max(local.len()) {
        let (theirs, ours) = (
            remote.get(at).copied().unwrap_or(0),
            local.get(at).copied().unwrap_or(0),
        );
        if theirs != ours {
            return theirs > ours;
        }
    }
    false
}

/// What the newest release calls itself, when the answer is readable.
async fn latest_release() -> Option<String> {
    let answer = reqwest::get(API).await.ok()?;
    if !answer.status().is_success() {
        return None;
    }
    let body = answer.text().await.ok()?;
    let value: serde_json::Value = serde_json::from_str(&body).ok()?;
    value.get("tag_name")?.as_str().map(|tag| tag.trim_start_matches('v').to_owned())
}

/// Looks for a newer release. Quiet is the look taken when the app opens: it only speaks up when
/// there is something to say; a check the user asked for always answers.
pub fn check(quiet: bool) {
    let local = version();
    let store = channel() == "微软商店";
    if store {
        if !quiet {
            go("ms-windows-store://downloadsandupdates");
        }
        return;
    }
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        let Some(remote) = latest_release().await else {
            if !quiet {
                post(&dispatcher, || {
                    let _ = crate::ui::shell::toast("检查更新失败：取不到最新版本");
                });
            }
            return;
        };
        if !newer(&remote, &local) {
            if !quiet {
                post(&dispatcher, move || {
                    let _ = crate::ui::shell::toast(&format!("已是最新版本 {local}"));
                });
            }
            return;
        }
        post(&dispatcher, move || {
            let _ = crate::ui::update_sheet::ask(&remote, REPO);
        });
    });
}
