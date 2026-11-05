//! Generates the XAML bindings of `crates/uwp`.

fn main() {
    windows_bindgen::bindgen([
        "--in",
        r"C:\Program Files (x86)\Windows Kits\10\UnionMetadata\10.0.22621.0\Windows.winmd",
        "--out",
        "../../crates/uwp/src/xaml.rs",
        "--minimal",
        "--flat",
        "--filter-file",
        "filter.txt",
        "--implement",
        "Windows.UI.Xaml.IApplicationOverrides",
        "--compose",
        "Windows.UI.Xaml.Application",
    ]);
}
