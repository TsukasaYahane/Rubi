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
    /// The path of the picked folder, once it has been read: asking Windows again on every page
    /// build is slow enough to be felt.
    static WHERE: RefCell<Option<String>> = const { RefCell::new(None) };
    /// The same for the folder offline copies are written into.
    static DOWNLOADS: RefCell<Option<String>> = const { RefCell::new(None) };
}

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
    kept(EXPORT_TOKEN)
}

/// The folder the user picked for offline copies, if any.
pub fn download_folder() -> Option<StorageFolder> {
    kept(DOWNLOAD_TOKEN)
}

fn kept(token: &str) -> Option<StorageFolder> {
    let list = StorageApplicationPermissions::FutureAccessList().ok()?;
    if !list.ContainsItem(token).ok()? {
        return None;
    }
    list.GetItemAsync(token).ok()?.join().ok()?.cast().ok()
}

/// Keeps `folder` as the one exports go to.
pub fn keep_folder(folder: &StorageFolder) -> Result<()> {
    let path = keep(EXPORT_TOKEN, folder)?;
    WHERE.with_borrow_mut(|slot| *slot = Some(path));
    Ok(())
}

/// Keeps `folder` as the one offline copies go into; the app has to be started again for the
/// core to write there.
pub fn keep_download_folder(folder: &StorageFolder) -> Result<()> {
    let path = keep(DOWNLOAD_TOKEN, folder)?;
    DOWNLOADS.with_borrow_mut(|slot| *slot = Some(path));
    Ok(())
}

fn keep(token: &str, folder: &StorageFolder) -> Result<String> {
    let item = folder.cast::<IStorageItem>()?;
    StorageApplicationPermissions::FutureAccessList()?.AddOrReplace(token, &item, "")?;
    Ok(item.Path()?.to_string())
}

/// Where exports go, as last read; `None` while nothing has been picked.
pub fn folder_path() -> Option<String> {
    if let Some(path) = WHERE.with_borrow(Clone::clone) {
        return Some(path);
    }
    let path = kept(EXPORT_TOKEN)?.cast::<IStorageItem>().ok()?.Path().ok()?.to_string();
    WHERE.with_borrow_mut(|slot| *slot = Some(path.clone()));
    Some(path)
}

/// Where offline copies go, as last read; `None` keeps them in the app's own folder.
pub fn download_path() -> Option<String> {
    if let Some(path) = DOWNLOADS.with_borrow(Clone::clone) {
        return Some(path);
    }
    let path = kept(DOWNLOAD_TOKEN)?.cast::<IStorageItem>().ok()?.Path().ok()?.to_string();
    DOWNLOADS.with_borrow_mut(|slot| *slot = Some(path.clone()));
    Some(path)
}
