//! Generates the XAML bindings of `crates/uwp`, with the few Win32 interfaces (Direct3D 11, DXGI,
//! composition interop) the video frames are drawn with. Both come from one run, so a Win32
//! interface that names a WinRT type gets the very type the rest of the bindings use.

fn main() {
    windows_bindgen::builder()
        .input(r"C:\Program Files (x86)\Windows Kits\10\UnionMetadata\10.0.22621.0\Windows.winmd")
        .input_bytes(windows_default::WIN32)
        .output("../../crates/uwp/src/xaml.rs")
        .minimal()
        .flat()
        .filter_file("filter.txt")
        .implement("Windows.UI.Xaml.IApplicationOverrides")
        .compose("Windows.UI.Xaml.Application")
        .write();
}
