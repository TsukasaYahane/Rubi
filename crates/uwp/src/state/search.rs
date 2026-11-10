//! What the search and UP pages say, without touching XAML.

use rubi_model::UpProfile;

use crate::state::library::{age, count};

/// A line under an UP's name in the results.
pub fn hit_line(fans: u64, videos: u64) -> String {
    format!("粉丝 {} · 投稿 {}", count(fans), count(videos))
}

/// A line under an UP's name on their page.
pub fn level_line(profile: &UpProfile) -> String {
    if profile.official.is_empty() {
        format!("Lv{}", profile.level)
    } else {
        format!("Lv{} · {}", profile.level, profile.official)
    }
}

/// A line under a 专栏's title.
pub fn article_line(views: u64, published_at: i64, now: i64) -> String {
    format!("{} 阅读 · {}", count(views), age(published_at, now))
}

/// A way to open a 专栏 by its number.
pub fn article_link(cvid: u64) -> String {
    format!("cv{cvid}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_read_well() {
        assert_eq!(hit_line(124_300, 320), "粉丝 12.4万 · 投稿 320");
        let profile = UpProfile { level: 6, ..UpProfile::default() };
        assert_eq!(level_line(&profile), "Lv6");
        let official = UpProfile { official: "知名 UP 主".into(), ..profile };
        assert_eq!(level_line(&official), "Lv6 · 知名 UP 主");
        assert_eq!(article_line(12_000, 1_800_000_000 - 7_200, 1_800_000_000), "1.2万 阅读 · 2 小时前");
        assert_eq!(article_link(42), "cv42");
    }
}
