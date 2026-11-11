//! The picture viewer's arithmetic: how a picture fits the window, which are 长图, where a zoom
//! keeps the point under the pointer, and what a saved picture is called. No XAML here.

/// Height over width from which a picture is a 长图: shown at the window's width and scrolled.
pub const LONG_RATIO: f64 = 3.0;
/// Most a picture is enlarged.
pub const MAX_ZOOM: f32 = 4.0;
/// Where a double click takes it.
pub const DOUBLE_ZOOM: f32 = 2.5;

/// How a picture of `ratio` (height over width) lies in a viewport `width` × `height`: the size of
/// the picture, and of the area the viewer scrolls (the viewport itself, or a 长图's full length).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub picture: (f64, f64),
    pub area: (f64, f64),
    pub long: bool,
}

pub fn fit(ratio: f64, (width, height): (f64, f64)) -> Fit {
    let (width, height) = (width.max(1.0), height.max(1.0));
    if ratio >= LONG_RATIO {
        let tall = width * ratio;
        return Fit { picture: (width, tall), area: (width, tall), long: true };
    }
    let ratio = if ratio > 0.0 { ratio } else { 1.0 };
    let shown = width.min(height / ratio);
    Fit { picture: (shown, shown * ratio), area: (width, height), long: false }
}

/// The scroll offsets that keep the point `at` (in the viewport) still while the zoom goes from
/// `from` to `to`, the offsets now being `offset`.
pub fn zoom_offsets(at: (f64, f64), offset: (f64, f64), from: f32, to: f32) -> (f64, f64) {
    let k = f64::from(to) / f64::from(from.max(0.01));
    (((at.0 + offset.0) * k - at.0).max(0.0), ((at.1 + offset.1) * k - at.1).max(0.0))
}

/// The picture as uploaded, without a size asked of the image server.
pub fn original(url: &str) -> &str {
    url.split('@').next().unwrap_or(url)
}

/// The name a saved picture takes: its own, from the address.
pub fn file_name(url: &str) -> String {
    let path = original(url).split('?').next().unwrap_or_default();
    let name = path.rsplit('/').next().unwrap_or_default();
    let safe: String = name.chars().filter(|c| !r#"\/:*?"<>|"#.contains(*c)).collect();
    if safe.contains('.') && safe.len() > 2 { safe } else { "picture.jpg".to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pictures_fit_the_window() {
        let wide = fit(0.5, (800.0, 600.0));
        assert_eq!(wide, Fit { picture: (800.0, 400.0), area: (800.0, 600.0), long: false });
        let tall = fit(2.0, (800.0, 600.0));
        assert_eq!(tall.picture, (300.0, 600.0));
        let long = fit(5.0, (800.0, 600.0));
        assert_eq!(long, Fit { picture: (800.0, 4000.0), area: (800.0, 4000.0), long: true });
        assert!(!fit(0.0, (800.0, 600.0)).long, "an unknown shape is shown square");
    }

    #[test]
    fn a_zoom_keeps_the_pointer_still() {
        assert_eq!(zoom_offsets((100.0, 50.0), (0.0, 0.0), 1.0, 2.0), (100.0, 50.0));
        assert_eq!(zoom_offsets((100.0, 50.0), (100.0, 50.0), 2.0, 1.0), (0.0, 0.0));
    }

    #[test]
    fn saved_pictures_keep_their_names() {
        let url = "https://i0.hdslb.com/bfs/new_dyn/abc123.jpg@240w_30q.jpg";
        assert_eq!(original(url), "https://i0.hdslb.com/bfs/new_dyn/abc123.jpg");
        assert_eq!(file_name(url), "abc123.jpg");
        assert_eq!(file_name("https://x/y/"), "picture.jpg");
    }
}
