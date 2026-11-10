//! What the 历史记录, 稍后再看 and 收藏夹 lists show, without touching XAML.

use rubi_model::{FavFolder, HistoryEntry, VideoCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shelf {
    History,
    Later,
    Favorites,
}

impl Shelf {
    pub const ALL: [Self; 3] = [Self::History, Self::Later, Self::Favorites];

    pub fn label(self) -> &'static str {
        match self {
            Self::History => "历史记录",
            Self::Later => "稍后再看",
            Self::Favorites => "收藏夹",
        }
    }
}

/// A video in one of the lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub bvid: String,
    pub aid: u64,
    pub title: String,
    pub owner: String,
    pub cover: String,
    pub line: String,
    pub finished: bool,
}

const FINISHED: i64 = -1;

impl Row {
    fn of(card: VideoCard, line: String, finished: bool) -> Self {
        Self {
            bvid: card.bvid,
            aid: card.aid,
            title: card.title,
            owner: card.owner.name,
            cover: card.cover,
            line,
            finished,
        }
    }

    pub fn watched(entry: HistoryEntry, now: i64) -> Self {
        let line = format!("{} · {}", progress(entry.progress_s), age(entry.viewed_at, now));
        Self::of(entry.card, line, entry.progress_s == FINISHED)
    }

    pub fn saved_for_later(entry: HistoryEntry) -> Self {
        let line = progress(entry.progress_s);
        Self::of(entry.card, line, entry.progress_s == FINISHED)
    }

    pub fn favorite(card: VideoCard, now: i64) -> Self {
        let line = format!("{} 播放 · {}", count(card.stats.view), age(card.pubdate, now));
        Self::of(card, line, false)
    }
}

pub fn progress(seconds: i64) -> String {
    match seconds {
        FINISHED => "已看完".into(),
        0 => "刚开始".into(),
        s => format!("看到 {}:{:02}", s / 60, s % 60),
    }
}

pub fn count(n: u64) -> String {
    if n >= 10_000 { format!("{:.1}万", n as f64 / 10_000.0) } else { n.to_string() }
}

/// How long ago `unix` was; a date once it is a week old.
pub fn age(unix: i64, now: i64) -> String {
    let gone = now - unix;
    match gone {
        _ if unix <= 0 => String::new(),
        ..3_600 => format!("{} 分钟前", (gone / 60).max(1)),
        3_600..86_400 => format!("{} 小时前", gone / 3_600),
        86_400..604_800 => format!("{} 天前", gone / 86_400),
        _ => date(unix),
    }
}

/// `unix` as a date in China Standard Time, as B站 shows it.
pub fn date(unix: i64) -> String {
    const CST: i64 = 8 * 3_600;
    // Howard Hinnant's civil-from-days.
    let days = (unix + CST).div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let of_era = days.rem_euclid(146_097);
    let year_of_era = (of_era - of_era / 1_460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 { shifted + 3 } else { shifted - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year}-{month:02}-{day:02}")
}

pub fn video_link(bvid: &str) -> String {
    format!("https://www.bilibili.com/video/{bvid}")
}

pub fn room_link(room_id: u64) -> String {
    format!("https://live.bilibili.com/{room_id}")
}

pub fn folder_line(folder: &FavFolder) -> String {
    let reach = if folder.private { "私密" } else { "公开" };
    let default = if folder.default { " · 默认" } else { "" };
    format!("{} 个视频 · {reach}{default}", folder.count)
}

/// From this wide a list has two columns.
const TWO_COLUMNS_FROM: f64 = 600.0;
const ROW_HEIGHT: f64 = 88.0;

/// Columns, then the width and height of a row, in a list `width` wide.
pub fn row_size(width: f64) -> (u32, f64, f64) {
    let columns = if width >= TWO_COLUMNS_FROM { 2 } else { 1 };
    (columns, (width / f64::from(columns)).floor().max(1.0), ROW_HEIGHT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_reads_as_on_android() {
        assert_eq!(progress(-1), "已看完");
        assert_eq!(progress(0), "刚开始");
        assert_eq!(progress(201), "看到 3:21");
        assert_eq!(progress(65), "看到 1:05");
    }

    #[test]
    fn age_steps_up_then_shows_a_date() {
        let now = 1_800_000_000;
        assert_eq!(age(now - 30, now), "1 分钟前");
        assert_eq!(age(now - 7_200, now), "2 小时前");
        assert_eq!(age(now - 3 * 86_400, now), "3 天前");
        assert_eq!(age(0, now), "");
        assert_eq!(age(1_700_000_000, now), "2023-11-15");
    }

    #[test]
    fn dates_fall_in_china_time() {
        assert_eq!(date(1_704_067_199 - 8 * 3_600), "2023-12-31");
        assert_eq!(date(1_704_067_200 - 8 * 3_600), "2024-01-01");
        assert_eq!(date(1_709_164_800 - 8 * 3_600), "2024-02-29");
    }

    #[test]
    fn counts_shorten_from_ten_thousand() {
        assert_eq!(count(9_999), "9999");
        assert_eq!(count(1_203_000), "120.3万");
    }

    #[test]
    fn a_wide_list_has_two_columns() {
        assert_eq!(row_size(500.0), (1, 500.0, 88.0));
        assert_eq!(row_size(750.0), (2, 375.0, 88.0));
    }

    #[test]
    fn only_a_finished_video_counts_as_watched() {
        let entry = |progress_s| HistoryEntry { card: VideoCard::default(), progress_s, viewed_at: 0 };
        assert!(Row::saved_for_later(entry(-1)).finished);
        assert!(!Row::saved_for_later(entry(30)).finished);
    }

    #[test]
    fn a_folder_says_what_it_is() {
        let folder = FavFolder { count: 3, private: true, default: true, ..FavFolder::default() };
        assert_eq!(folder_line(&folder), "3 个视频 · 私密 · 默认");
    }
}
