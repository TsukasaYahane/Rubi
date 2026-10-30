//! The 空降 adapter: recognises "jump to" danmaku, clusters them by target time
//! (±5 s) and keeps only the earliest-sent one per cluster.

use std::sync::OnceLock;

use regex::Regex;
use rubi_model::{Danmaku, JumpMark};

/// Targets within this distance of a cluster's first target join that cluster.
pub const CLUSTER_WINDOW_MS: u32 = 5_000;
/// A bare timestamp must point at least this far ahead of where it appears.
pub const BARE_MIN_LEAD_MS: u32 = 10_000;

const TIME: &str = r"(?P<h>\d{1,2})[:：](?P<hm>\d{2})[:：](?P<hs>\d{2})|(?P<m>\d{1,3})[:：](?P<s>\d{2})|(?P<cm>\d{1,3})\s*分钟?\s*(?:(?P<cs>\d{1,2})\s*秒?)?|(?P<dm>\d{1,3})[.．](?P<ds>\d{2})";

fn keyword_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new("空降|跳到|跳转到").unwrap())
}

fn time_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(TIME).unwrap())
}

fn bare_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!("^(?:{TIME})$")).unwrap())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpKind {
    /// Has a 空降 / 跳到 keyword; the time may be anywhere in the text.
    Keyword,
    /// The whole danmaku is just a timestamp.
    Bare,
}

/// Returns the jump target in ms if `content` is a 空降 danmaku.
/// `duration_ms` of 0 means the duration is unknown and is not checked.
pub fn parse_jump(content: &str, progress_ms: u32, duration_ms: u32) -> Option<(u32, JumpKind)> {
    let text = normalize_digits(content);
    let within = |t: u32| duration_ms == 0 || t <= duration_ms;

    if keyword_re().is_match(&text) {
        let target = time_re().captures(&text).and_then(|c| captured_ms(&c))?;
        return (target > 0 && within(target)).then_some((target, JumpKind::Keyword));
    }

    let core = text.trim_matches(|c: char| !c.is_alphanumeric());
    let target = bare_re().captures(core).and_then(|c| captured_ms(&c))?;
    (target >= progress_ms.saturating_add(BARE_MIN_LEAD_MS) && within(target)).then_some((target, JumpKind::Bare))
}

fn captured_ms(c: &regex::Captures) -> Option<u32> {
    let num = |name: &str| c.name(name).and_then(|m| m.as_str().parse::<u32>().ok());
    let (h, m, s) = if let Some(h) = num("h") {
        (h, num("hm")?, num("hs")?)
    } else if let Some(m) = num("m") {
        (0, m, num("s")?)
    } else if let Some(m) = num("cm") {
        (0, m, num("cs").unwrap_or(0))
    } else {
        (0, num("dm")?, num("ds")?)
    };
    if s >= 60 || (h > 0 && m >= 60) {
        return None;
    }
    Some((h * 3600 + m * 60 + s) * 1000)
}

fn normalize_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '０'..='９' => char::from_u32(c as u32 - '０' as u32 + '0' as u32).unwrap_or(c),
            _ => c,
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct Merged {
    /// Ids of jump danmaku removed because an earlier one covers the same target.
    pub dropped: Vec<u64>,
    /// One mark per cluster, sorted by target.
    pub marks: Vec<JumpMark>,
}

pub fn merge(dms: &[Danmaku], duration_ms: u32) -> Merged {
    let mut jumps: Vec<(u32, &Danmaku)> =
        dms.iter().filter_map(|d| parse_jump(&d.content, d.progress_ms, duration_ms).map(|(t, _)| (t, d))).collect();
    jumps.sort_by_key(|(t, d)| (*t, d.ctime, d.id));

    let mut out = Merged::default();
    let mut i = 0;
    while i < jumps.len() {
        let anchor = jumps[i].0;
        let end = jumps[i..].iter().position(|(t, _)| t - anchor > CLUSTER_WINDOW_MS).map_or(jumps.len(), |p| i + p);
        let cluster = &jumps[i..end];
        let (target, keep) = cluster.iter().min_by_key(|(_, d)| (d.ctime, d.progress_ms, d.id)).copied().unwrap();
        out.marks.push(JumpMark { target_ms: target, label: format_time(target), source_id: keep.id });
        out.dropped.extend(cluster.iter().filter(|(_, d)| d.id != keep.id).map(|(_, d)| d.id));
        i = end;
    }
    out
}

pub fn format_time(ms: u32) -> String {
    let s = ms / 1000;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUR: u32 = 30 * 60 * 1000;

    fn t(content: &str, at_s: u32) -> Option<u32> {
        parse_jump(content, at_s * 1000, DUR).map(|(ms, _)| ms / 1000)
    }

    #[test]
    fn keyword_jumps() {
        assert_eq!(t("空降 05:32", 0), Some(332));
        assert_eq!(t("空降到5:32", 0), Some(332));
        assert_eq!(t("5:32 空降", 0), Some(332));
        assert_eq!(t("跳到 1:02:03", 0), None, "beyond the 30 min duration");
        assert_eq!(parse_jump("跳到 1:02:03", 0, 0).map(|x| x.0), Some(3_723_000));
        assert_eq!(t("空降 5分32秒", 0), Some(332));
        assert_eq!(t("空降5分", 0), Some(300));
        assert_eq!(t("空降 5.32", 0), Some(332));
        assert_eq!(t("空降 ０５：３２", 0), Some(332), "full-width digits and colon");
        assert_eq!(t("空降到这里 10:00", 900), Some(600), "keyword jumps may point backwards");
    }

    #[test]
    fn keyword_without_time_is_not_a_jump() {
        assert_eq!(t("空降成功", 0), None);
        assert_eq!(t("空降的同学举手", 0), None);
    }

    #[test]
    fn bare_timestamps() {
        assert_eq!(t("05:32", 0), Some(332));
        assert_eq!(t("5:32。", 0), Some(332));
        assert_eq!(t("【05:32】", 0), Some(332));
        assert_eq!(t("5分32秒", 0), Some(332));
        assert_eq!(t("9.50", 0), Some(590));
    }

    #[test]
    fn bare_timestamps_must_point_forward_and_inside() {
        assert_eq!(t("05:32", 330), None, "only 2 s ahead");
        assert_eq!(t("05:32", 322), Some(332), "exactly 10 s ahead");
        assert_eq!(t("01:00", 120), None, "points backwards");
        assert_eq!(t("45:00", 0), None, "beyond duration");
    }

    #[test]
    fn bare_requires_the_whole_text() {
        assert_eq!(t("回到 03:10 那里", 0), None);
        assert_eq!(t("5:32 太好笑了", 0), None);
        assert_eq!(t("5:75", 0), None, "seconds out of range");
    }

    fn dm(id: u64, content: &str, progress_s: u32, ctime: i64) -> Danmaku {
        Danmaku { id, content: content.into(), progress_ms: progress_s * 1000, ctime, ..Default::default() }
    }

    #[test]
    fn merges_clusters_keeping_earliest_sent() {
        let dms = vec![
            dm(1, "空降 05:32", 10, 50),
            dm(2, "05:30", 20, 40),
            dm(3, "空降5:34", 30, 60),
            dm(4, "空降 05:38", 40, 10), // 8 s after the anchor 05:30 -> new cluster
            dm(5, "12:48", 100, 70),
            dm(6, "哈哈哈", 1, 1),
        ];
        let m = merge(&dms, DUR);
        let marks: Vec<(String, u64)> = m.marks.iter().map(|j| (j.label.clone(), j.source_id)).collect();
        assert_eq!(marks, vec![("05:30".into(), 2), ("05:38".into(), 4), ("12:48".into(), 5)]);
        let mut dropped = m.dropped.clone();
        dropped.sort();
        assert_eq!(dropped, vec![1, 3]);
    }
}
