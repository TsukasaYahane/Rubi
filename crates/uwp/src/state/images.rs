//! B站's image servers send a picture at the size asked: `…jpg@320w.jpg` comes back 320 px wide,
//! a small fraction of the original's bytes. Lists show a tiny, rough copy first and then one
//! sharp at the size shown.

/// The address of `url` `width` px wide, at `quality` (1–100) if given; `None` for pictures not
/// on B站's servers and for animated ones, which are taken whole.
pub fn sized(url: &str, width: u32, quality: Option<u32>) -> Option<String> {
    if !url.contains(".hdslb.com/") {
        return None;
    }
    let base = url.split(['@', '?']).next()?;
    let lower = base.to_ascii_lowercase();
    if lower.ends_with(".gif") {
        return None;
    }
    let ext = if lower.ends_with(".png") { "png" } else { "jpg" };
    Some(match quality {
        Some(q) => format!("{base}@{width}w_{q}q.{ext}"),
        None => format!("{base}@{width}w.{ext}"),
    })
}

/// The sharp copy for a picture decoded `decode_width` px wide: twice that, for high-DPI screens.
pub fn sharp_width(decode_width: i32) -> u32 {
    (decode_width.max(1) as u32 * 2).min(2048)
}

/// The rough copy shown while the sharp one comes.
pub fn rough_width(decode_width: i32) -> u32 {
    (decode_width.max(1) as u32 / 4).clamp(48, 160)
}

/// Quality of the rough copy.
pub const ROUGH_QUALITY: u32 = 30;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilibili_pictures_come_at_a_size() {
        let cover = "https://i1.hdslb.com/bfs/archive/abc.jpg";
        assert_eq!(sized(cover, 640, None).as_deref(), Some("https://i1.hdslb.com/bfs/archive/abc.jpg@640w.jpg"));
        assert_eq!(sized(cover, 96, Some(30)).as_deref(), Some("https://i1.hdslb.com/bfs/archive/abc.jpg@96w_30q.jpg"));
        assert_eq!(
            sized("https://i0.hdslb.com/bfs/face/x.PNG@100w.webp", 64, None).as_deref(),
            Some("https://i0.hdslb.com/bfs/face/x.PNG@64w.png"),
            "an earlier size is replaced and transparency kept"
        );
    }

    #[test]
    fn others_are_taken_whole() {
        assert_eq!(sized("https://i0.hdslb.com/bfs/new_dyn/a.gif", 320, None), None);
        assert_eq!(sized("https://example.com/a.jpg", 320, None), None);
    }

    #[test]
    fn widths_keep_to_their_ranges() {
        assert_eq!(sharp_width(320), 640);
        assert_eq!(sharp_width(4000), 2048);
        assert_eq!(rough_width(320), 80);
        assert_eq!(rough_width(40), 48);
    }
}
