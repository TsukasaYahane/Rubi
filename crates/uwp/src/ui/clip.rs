//! Watching the clipboard for a video address: Windows raises Clipboard.ContentChanged on every
//! change, so only the newest item is looked at, never the history.

use std::sync::Mutex;

use windows_core::{IInspectable, Interface, Ref};

use crate::ui::keep;
use crate::xaml::{Clipboard, IDataPackageView};

static LAST: Mutex<String> = Mutex::new(String::new());

/// Starts watching, once; a change that holds a video address opens that video.
pub fn watch() -> windows_core::Result<()> {
    static STARTED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if STARTED.set(()).is_err() {
        return Ok(());
    }
    keep(Clipboard::ContentChanged(move |_: Ref<IInspectable>, _: Ref<IInspectable>| {
        // Reading needs the clipboard's own thread, so it is done off the UI thread.
        std::thread::spawn(|| {
            let Some(text) = current_text() else { return };
            if text.trim().is_empty() {
                return;
            }
            {
                let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
                if *last == text {
                    return;
                }
                *last = text.clone();
            }
            if let Some(bvid) = crate::ui::search::video_in(&text) {
                let _ = crate::ui::search::open_video(&bvid);
            }
        });
    })?);
    Ok(())
}

/// The clipboard's text, if it has any.
fn current_text() -> Option<String> {
    let view = Clipboard::GetContent().ok()?;
    let view = view.cast::<IDataPackageView>().ok()?;
    let operation = view.GetTextAsync().ok()?;
    // Waited for from a worker thread: the UI thread must not block on the clipboard.
    for _ in 0..100 {
        if let Ok(text) = operation.GetResults() {
            return Some(text.to_string_lossy());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    None
}
