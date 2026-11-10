//! What the 影视 pages say and how they lay out, without touching XAML.

/// A poster corner that means money: red, where others are cobalt.
pub fn paid(badge: &str) -> bool {
    badge.contains("会员") || badge.contains("付费") || badge.contains("点播")
}

/// The narrowest a poster gets before a column goes.
const POSTER_MIN: f64 = 150.0;
const POSTER_COLUMNS: (u32, u32) = (3, 8);
/// Room under a poster for its three lines.
const POSTER_TEXT: f64 = 64.0;
const POSTER_GAP: f64 = 12.0;

/// Columns, cell width and cell height for posters in `width`.
pub fn poster_cell(width: f64) -> (u32, f64, f64) {
    let columns = ((width / POSTER_MIN).floor() as u32).clamp(POSTER_COLUMNS.0, POSTER_COLUMNS.1);
    let cell = (width / f64::from(columns)).floor().max(1.0);
    let poster = ((cell - POSTER_GAP).max(1.0) * 4.0 / 3.0).floor();
    (columns, cell, poster + POSTER_TEXT)
}

/// A poster at `width` pixels: B站's image server scales it down, as the originals are PNGs of a
/// couple of megabytes that a wall of them takes long to fetch.
pub fn poster_url(url: &str, width: u32) -> String {
    if url.contains("hdslb.com") && !url.contains('@') {
        format!("{url}@{width}w_{}h_1c.jpg", width * 4 / 3)
    } else {
        url.to_owned()
    }
}

/// The line under a poster's title: score and how far it has come.
pub fn season_line(score: &str, index_show: &str) -> String {
    match (score.is_empty(), index_show.is_empty()) {
        (false, false) => format!("★ {score} · {index_show}"),
        (false, true) => format!("★ {score}"),
        _ => index_show.to_owned(),
    }
}

/// An episode's button: 「第3话 魔法使的考验」, or its own title when that is not a number.
pub fn episode_label(title: &str, long_title: &str) -> String {
    let head = if !title.is_empty() && title.chars().all(|c| c.is_ascii_digit()) {
        format!("第{title}话")
    } else {
        title.to_owned()
    };
    if long_title.is_empty() { head } else { format!("{head} {long_title}") }
}

/// Episode columns beside the poster on a wide window, under it otherwise.
pub fn episode_columns(width: f64) -> u32 {
    if width >= 520.0 { 3 } else { 2 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_badges_are_told_apart() {
        assert!(paid("大会员"));
        assert!(paid("付费"));
        assert!(!paid("独家"));
        assert!(!paid(""));
    }

    #[test]
    fn posters_take_what_width_allows() {
        assert_eq!(poster_cell(300.0).0, 3);
        let (columns, cell, height) = poster_cell(900.0);
        assert_eq!((columns, cell), (6, 150.0));
        assert_eq!(height, ((150.0_f64 - 12.0) * 4.0 / 3.0).floor() + 64.0);
        assert_eq!(poster_cell(5000.0).0, 8);
    }

    #[test]
    fn posters_are_asked_for_small() {
        assert_eq!(
            poster_url("https://i0.hdslb.com/bfs/b/a.png", 300),
            "https://i0.hdslb.com/bfs/b/a.png@300w_400h_1c.jpg"
        );
        assert_eq!(poster_url("https://i0.hdslb.com/a.png@100w.jpg", 300), "https://i0.hdslb.com/a.png@100w.jpg");
        assert_eq!(poster_url("https://example.com/a.png", 300), "https://example.com/a.png");
    }

    #[test]
    fn lines_read_well() {
        assert_eq!(season_line("9.6", "全25话"), "★ 9.6 · 全25话");
        assert_eq!(season_line("", "更新至第4话"), "更新至第4话");
        assert_eq!(season_line("9.1", ""), "★ 9.1");
        assert_eq!(episode_label("3", "魔法使的考验"), "第3话 魔法使的考验");
        assert_eq!(episode_label("正片", ""), "正片");
        assert_eq!(episode_label("SP", "特别篇"), "SP 特别篇");
    }
}
