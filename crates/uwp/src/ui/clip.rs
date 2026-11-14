//! Watching the clipboard for a video address: Windows raises Clipboard.ContentChanged on every
//! change, so only the newest item is looked at, never the history.

use std::sync::Mutex;

use windows_core::{IInspectable, Interface, Ref};

use crate::ui::keep;
use crate::xaml::{Clipboard, IDataPackageView};

static LAST: Mutex<String> = Mutex::new(String::new());

/// Looks at the clipboard's newest item once: a video address in it opens that video.
pub fn check() {
    handle(current_text());
}

/// Starts watching, once; a change that holds a video address opens that video.
pub fn watch() -> windows_core::Result<()> {
    static STARTED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if STARTED.set(()).is_err() {
        return Ok(());
    }
    keep(Clipboard::ContentChanged(move |_: Ref<IInspectable>, _: Ref<IInspectable>| {
        // The text is local, so it is read here first; a worker only if that gives nothing.
        let here = current_text();
        if here.is_none() {
            std::thread::spawn(|| handle(current_text()));
            return;
        }
        handle(here);
    })?);
    Ok(())
}

fn handle(text: Option<String>) {
    // TEMP-PROBE: says the event fired and what came with it.
    match &text {
        Some(t) => crate::ui::shell::toast(&format!("剪贴板：{}", t.chars().take(24).collect::<String>())),
        None => crate::ui::shell::toast("剪贴板：读不到文本"),
    }
    let Some(text) = text else { return };
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
