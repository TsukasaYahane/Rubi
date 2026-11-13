//! A sheet over the window, made once and reused: 被过滤内容 and 为什么推荐 sit here.

use windows_collections::IVector;
use windows_core::{Interface, Result};

use crate::ui::shell::shell;
use crate::xaml::{IFrameworkElement, IUIElement, UIElement, Visibility};

/// Shows `root` over the window, replacing the sheet that was there.
pub fn show(root: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let host = shell.viewer_host()?;
    let children = host.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    children.Append(&root.cast::<UIElement>()?)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)
}

/// Takes the sheet away.
pub fn hide() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let host = shell.viewer_host()?;
    host.Children()?.cast::<IVector<UIElement>>()?.Clear()?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)
}
