//! 导出: what a finished 离线缓存 copy is also written out as, and where. Kept in memory,
//! since a download does not outlive the app that started it anyway.

use std::cell::RefCell;
use std::collections::HashMap;

use windows_core::{Interface, Result};

use rubi_core::ExportFormat;

use crate::xaml::{IStorageItem, StorageApplicationPermissions, StorageFolder};

/// The name the picked folder is kept under, in the list Windows lets an app keep across runs.
const EXPORT_TOKEN: &str = "rubi-export";
/// The same, for the folder 离线缓存 writes into.
const DOWNLOAD_TOKEN: &str = "rubi-downloads";

thread_local! {
    static WANTED: RefCell<HashMap<String, ExportFormat>> = RefCell::default();
}

/// The path of the picked folder, once it has been read: asking Windows again on every page build
/// is slow enough to be felt. Both live outside the thread local block because the picker answers
/// on whichever thread Windows chose, and the lines that read them are drawn on the ui thread.
static WHERE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
/// The same for the folder offline copies are written into.
static DOWNLOADS: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Notes that `bvid` is to be written out as `format` once it finishes.
pub fn remember(bvid: &str, format: ExportFormat) {
    WANTED.with_borrow_mut(|map| {
        map.insert(bvid.to_owned(), format);
    });
}

/// Takes what was noted for `bvid`, if anything: a note is used once.
pub fn take(bvid: &str) -> Option<ExportFormat> {
    WANTED.with_borrow_mut(|map| map.remove(bvid))
}

/// The folder the user picked for exports, while it is still one Windows will let us open.
pub fn folder() -> Option<StorageFolder> {
    folder_for(&WHERE, EXPORT_TOKEN)
}

/// The folder the user picked for offline copies, if any.
pub fn download_folder() -> Option<StorageFolder> {
    folder_for(&DOWNLOADS, DOWNLOAD_TOKEN)
}

/// Opens the folder the path says, not whichever entry the token happens to name: the two lines
/// each show a path, and the thing a press opens has to be that path.
fn folder_for(slot: &std::sync::Mutex<Option<String>>, token: &str) -> Option<StorageFolder> {
    if let Some(path) = read_from(slot)
        && let Ok(operation) = crate::xaml::StorageFolder::GetFolderFromPathAsync(&path)
        && let Ok(folder) = operation.join()
    {
        return Some(folder);
    }
    kept(token)
}


/// The picker answers on whichever thread Windows chose, so what it learned cannot live in a
/// thread local: the line that reads it is drawn on the ui thread.
fn keep_in(slot: &std::sync::Mutex<Option<String>>, path: String) {
    if let Ok(mut held) = slot.lock() {
        *held = Some(path);
    }
}

fn read_from(slot: &std::sync::Mutex<Option<String>>) -> Option<String> {
    slot.lock().ok().and_then(|held| held.clone())
}

fn kept(token: &str) -> Option<StorageFolder> {
    let list = StorageApplicationPermissions::FutureAccessList().ok()?;
    if !list.ContainsItem(token).ok()? {
        crate::ui::trace(&format!("{token} is not in the list"));
        return None;
    }
    let folder: StorageFolder = list.GetItemAsync(token).ok()?.join().ok()?.cast().ok()?;
    // Said out loud: the two tokens have to answer with two different folders, and the log is
    // the only place that shows which one an entry actually holds.
    let path = folder.cast::<IStorageItem>().ok().and_then(|item| item.Path().ok());
    crate::ui::trace(&format!("{token} opens {:?}", path.map(|p| p.to_string())));
    Some(folder)
}

/// Keeps `folder` as the one exports go to.
pub fn keep_folder(folder: &StorageFolder) -> Result<()> {
    let path = keep(EXPORT_TOKEN, folder)?;
    keep_in(&WHERE, path);
    Ok(())
}

/// Keeps `folder` as the one offline copies go into; the app has to be started again for the
/// core to write there.
pub fn keep_download_folder(folder: &StorageFolder) -> Result<()> {
    let path = keep(DOWNLOAD_TOKEN, folder)?;
    keep_in(&DOWNLOADS, path);
    Ok(())
}

fn keep(token: &str, folder: &StorageFolder) -> Result<String> {
    let item = folder.cast::<IStorageItem>()?;
    StorageApplicationPermissions::FutureAccessList()?.AddOrReplace(token, &item, "")?;
    let path = item.Path()?.to_string();
    crate::ui::trace(&format!("kept {token} -> {path}"));
    Ok(path)
}

/// Where exports go, as last read; `None` while nothing has been picked.
pub fn folder_path() -> Option<String> {
    if let Some(path) = read_from(&WHERE) {
        return Some(path);
    }
    let path = kept(EXPORT_TOKEN)?.cast::<IStorageItem>().ok()?.Path().ok()?.to_string();
    keep_in(&WHERE, path.clone());
    Some(path)
}

/// Where offline copies go, as last read; `None` keeps them in the app's own folder.
pub fn download_path() -> Option<String> {
    if let Some(path) = read_from(&DOWNLOADS) {
        return Some(path);
    }
    let path = kept(DOWNLOAD_TOKEN)?.cast::<IStorageItem>().ok()?.Path().ok()?.to_string();
    keep_in(&DOWNLOADS, path.clone());
    Some(path)
}
