//! The UWP app. XAML has no compiler here: pages are XAML strings loaded at run time, behaviour
//! is attached in code over the bindings in `xaml`, and everything the app knows comes from
//! `rubi-core`. [`state`] holds what does not touch XAML, so the WinUI version can share it.

#![cfg_attr(target_vendor = "uwp", windows_subsystem = "windows")]

#[cfg(target_vendor = "uwp")]
mod app;
#[cfg(target_vendor = "uwp")]
mod backend;
#[cfg(target_vendor = "uwp")]
mod session;
#[cfg_attr(not(target_vendor = "uwp"), allow(dead_code))]
mod state;
#[cfg(target_vendor = "uwp")]
mod ui;
#[cfg(target_vendor = "uwp")]
#[allow(dead_code, non_snake_case, non_camel_case_types, clippy::all)]
#[rustfmt::skip]
mod xaml;

#[cfg(target_vendor = "uwp")]
fn main() -> windows_core::Result<()> {
    app::run()
}

/// Off a UWP target there is nothing to run; `state` still builds and tests.
#[cfg(not(target_vendor = "uwp"))]
fn main() {}
