//! 导出: what a finished 离线缓存 copy is also written out as. Kept in memory, since a
//! download does not outlive the app that started it anyway.

use std::cell::RefCell;
use std::collections::HashMap;

use rubi_core::ExportFormat;

thread_local! {
    static WANTED: RefCell<HashMap<String, ExportFormat>> = RefCell::default();
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
