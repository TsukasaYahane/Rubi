//! A player page's picture filling the screen, or only the window (窗口全屏), for the video
//! page and the live room alike.

use std::cell::Cell;

use windows_core::Result;

use crate::ui::shell::shell;

/// The glyphs of the 窗口全屏 button: to enter, to leave.
const WINDOW_GLYPHS: (&str, &str) = ("\u{E9A6}", "\u{E73F}");

#[derive(Default)]
pub struct Filling {
    on: Cell<bool>,
    window: Cell<bool>,
}

impl Filling {
    /// The page around the picture is folded away, either way.
    pub fn on(&self) -> bool {
        self.on.get()
    }

    /// The screen itself is filled, not only the window.
    pub fn screen(&self) -> bool {
        self.on.get() && !self.window.get()
    }

    /// The 窗口全屏 button's glyph for the present state.
    pub fn window_glyph(&self) -> &'static str {
        if self.on.get() && self.window.get() { WINDOW_GLYPHS.1 } else { WINDOW_GLYPHS.0 }
    }

    /// Enters the way asked, or leaves it when already in it. Whether anything changed.
    pub fn toggle(&self, window: bool) -> Result<bool> {
        let on = !(self.on.get() && self.window.get() == window);
        self.set(on, window)
    }

    /// Leaves whichever it is in. Whether anything changed.
    pub fn leave(&self) -> Result<bool> {
        self.set(false, false)
    }

    /// The screen was left by the system's own control while filled: follows it. Whether
    /// anything changed.
    pub fn follow_system(&self) -> Result<bool> {
        if self.screen() && !shell().is_some_and(|s| s.is_filling_screen()) { self.leave() } else { Ok(false) }
    }

    /// Fills the screen, or only the window when `window`; or neither. Whether anything changed.
    fn set(&self, on: bool, window: bool) -> Result<bool> {
        let window = on && window;
        let (was, was_window) = (self.on.get(), self.window.get());
        if was == on && was_window == window {
            return Ok(false);
        }
        self.on.set(on);
        self.window.set(window);
        let Some(shell) = shell() else { return Ok(true) };
        match (on, window) {
            (true, true) => {
                if shell.is_filling_screen() {
                    shell.fill_screen(false)?;
                }
                shell.immersive(true)?;
            }
            (true, false) => shell.fill_screen(true)?,
            (false, _) if was_window => shell.immersive(false)?,
            (false, _) => shell.fill_screen(false)?,
        }
        Ok(true)
    }
}
