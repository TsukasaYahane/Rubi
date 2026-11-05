//! The login QR code as vector path data: XAML draws it at any size, with no bitmap to decode.

use qrcode::{Color, QrCode};

/// Modules per side, and one `Path` data string of the dark modules; `None` when `text` does
/// not fit a QR code.
pub fn path_data(text: &str) -> Option<(usize, String)> {
    let code = QrCode::new(text.as_bytes()).ok()?;
    let side = code.width();
    let mut data = String::new();
    for (i, color) in code.to_colors().iter().enumerate() {
        if *color == Color::Dark {
            data.push_str(&format!("M{} {}h1v1h-1z", i % side, i / side));
        }
    }
    Some((side, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_the_dark_modules() {
        let (side, data) = path_data("https://passport.bilibili.com/qrcode/h5/login?qrcode_key=abc").unwrap();
        assert!(side >= 21);
        assert!(data.starts_with("M0 0h1v1h-1z"), "the finder pattern starts at the corner");
    }
}
