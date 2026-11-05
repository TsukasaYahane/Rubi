//! The UWP app. XAML has no compiler here: controls are built in code over the bindings in
//! [`xaml`], and everything the app knows comes from `rubi-core`.

#![cfg_attr(target_vendor = "uwp", windows_subsystem = "windows")]

#[cfg(target_vendor = "uwp")]
mod app;
#[cfg(target_vendor = "uwp")]
#[allow(dead_code, non_snake_case, non_camel_case_types, clippy::all)]
mod xaml;

#[cfg(target_vendor = "uwp")]
fn main() -> windows_core::Result<()> {
    app::run()
}

/// Off a UWP target there is nothing to run; the crate only has to build with the workspace.
#[cfg(not(target_vendor = "uwp"))]
fn main() {}
