//! A sheet over the window, made once and reused: 被过滤内容 and 为什么推荐 sit here.

use windows_collections::IVector;
use windows_core::{Interface, Result};

use std::cell::RefCell;

use crate::ui::shell::shell;
use crate::xaml::{IFrameworkElement, IUIElement, UIElement, Visibility};

thread_local! {
    /// The sheet on show, kept so a background answer can reach it from the ui thread.
    static SHOWN: RefCell<Option<IFrameworkElement>> = const { RefCell::new(None) };
}

/// The sheet on show, for code that runs after the page was built and cannot carry the element.
pub fn current() -> Option<IFrameworkElement> {
    SHOWN.with_borrow(Clone::clone)
}

/// Shows `root` over the window, replacing the sheet that was there.
pub fn show(root: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let host = shell.viewer_host()?;
    let children = host.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    children.Append(&root.cast::<UIElement>()?)?;
    SHOWN.with_borrow_mut(|slot| *slot = Some(root.clone()));
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)
}

/// Takes the sheet away.
pub fn hide() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let host = shell.viewer_host()?;
    host.Children()?.cast::<IVector<UIElement>>()?.Clear()?;
    SHOWN.with_borrow_mut(|slot| *slot = None);
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)
}
