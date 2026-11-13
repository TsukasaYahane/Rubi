//! What the settings page offers, and what changing it writes. The core owns its settings as one
//! JSON object and takes patches of just what changed; a few belong to this app alone.

use rubi_core::LiveFilter;

use crate::state::danmaku::Look;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Look,
    Play,
    Filter,
    Recommend,
    Account,
    About,
}

impl Category {
    pub const ALL: [Self; 6] = [Self::Look, Self::Play, Self::Filter, Self::Recommend, Self::Account, Self::About];

    pub fn label(self) -> &'static str {
        match self {
            Self::Look => "外观",
            Self::Play => "播放",
            Self::Filter => "过滤",
            Self::Recommend => "推荐",
            Self::Account => "账号",
            Self::About => "关于",
        }
    }
}

/// A switch among the core's filter rules.
pub struct Rule {
    pub key: &'static str,
    pub title: &'static str,
    pub note: &'static str,
}

const fn rule(key: &'static str, title: &'static str, note: &'static str) -> Rule {
    Rule { key, title, note }
}

pub const FILTERS: [Rule; 8] = [
    rule("promotion", "必火推广", "隐藏付费加推的视频"),
    rule("ad", "广告卡片", "推荐流、列表、搜索中的广告"),
    rule("banner", "首页横幅与活动页", ""),
    rule("goods", "视频页商品卡片", ""),
    rule("pinned_comment", "置顶带货评论", ""),
    rule("commercial_penalty", "商业合作稿件降权", "默认关闭，不惩罚 UP 主商单"),
    rule("notice", "营销私信与推广通知", ""),
    rule("watched", "推荐中隐藏已看过", ""),
];

pub const DANMAKU: [Rule; 3] = [
    rule("command_danmaku", "过滤互动指令弹幕", "投票、关注、三连、链接卡片"),
    rule("show_advanced_danmaku", "显示高级弹幕", "mode 7 定位弹幕"),
    rule("merge_jump", "合并空降弹幕", "前后 5 秒内只保留最早一条"),
];

pub const FACTORS: [(&str, &str); 8] = [
    ("interest", "兴趣匹配"),
    ("follow", "关注 UP"),
    ("freshness", "新鲜度"),
    ("topic", "话题度"),
    ("quality", "内容质量"),
    ("explore", "探索"),
    ("dislike", "不感兴趣"),
    ("repeat_exposure", "重复曝光"),
];

/// Largest weight the sliders offer.
pub const WEIGHT_MAX: f64 = 0.6;

pub const QUALITIES: [(u32, &str); 3] = [(80, "1080P"), (64, "720P"), (32, "480P")];

const DEFAULT_VOLUME: u8 = 70;

/// This app's own preferences, kept in a small file beside the panes' widths.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prefs {
    /// Words alone, without line icons.
    pub words: bool,
    pub pause_on_blur: bool,
    pub shortcuts: bool,
    /// Volume of the players, 0 to 100.
    pub volume: u8,
    pub danmaku: Look,
    /// What live rooms hold back.
    pub live_filter: LiveFilter,
    /// A narrow window shows the feed in one column instead of two.
    pub single_column: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            words: false, pause_on_blur: true, shortcuts: true, volume: DEFAULT_VOLUME,
            danmaku: Look::default(),
            live_filter: LiveFilter::default(),
            single_column: false,
        }
    }
}

impl Prefs {
    /// Reads what [`to_text`](Self::to_text) wrote; what is missing keeps its default.
    pub fn parse(text: &str) -> Self {
        let mut prefs = Self::default();
        for pair in text.split_whitespace() {
            let Some((key, value)) = pair.split_once('=') else { continue };
            let on = value == "1";
            match key {
                "words" => prefs.words = on,
                "pause_on_blur" => prefs.pause_on_blur = on,
                "shortcuts" => prefs.shortcuts = on,
                "volume" | "live_volume" => prefs.volume = value.parse().map_or(DEFAULT_VOLUME, |v: u8| v.min(100)),
                "dm_area" => prefs.danmaku.area = value.parse().map_or(prefs.danmaku.area, |v: f64| v.clamp(0.1, 1.0)),
                "dm_speed" => {
                    prefs.danmaku.speed = value.parse().map_or(prefs.danmaku.speed, |v: f64| v.clamp(0.3, 3.0))
                }
                "dm_size" => {
                    prefs.danmaku.size = value.parse().map_or(prefs.danmaku.size, |v: f64| v.clamp(12.0, 40.0))
                }
                "dm_opacity" => {
                    prefs.danmaku.opacity = value.parse().map_or(prefs.danmaku.opacity, |v: f64| v.clamp(0.2, 1.0));
                }
                "live_filter" => {
                    if let Ok(bits) = value.parse::<u8>() {
                        prefs.live_filter =
                            LiveFilter { gifts: bits & 1 != 0, popups: bits & 2 != 0, effects: bits & 4 != 0 };
                    }
                }
                "dm_kinds" => {
                    if let Ok(bits) = value.parse() {
                        prefs.danmaku = prefs.danmaku.with_kinds(bits);
                    }
                }
                _ => {}
            }
        }
        prefs
    }

    pub fn to_text(self) -> String {
        let dm = self.danmaku;
        format!(
            "words={} pause_on_blur={} shortcuts={} volume={} dm_area={} dm_speed={} dm_size={} dm_opacity={} dm_kinds={} live_filter={}",
            u8::from(self.words),
            u8::from(self.pause_on_blur),
            u8::from(self.shortcuts),
            self.volume,
            dm.area,
            dm.speed,
            dm.size,
            dm.opacity,
            dm.kinds(),
            u8::from(self.live_filter.gifts) | u8::from(self.live_filter.popups) << 1 | u8::from(self.live_filter.effects) << 2
        )
    }
}

pub fn rule_patch(key: &str, on: bool) -> Value {
    json!({ "rules": { "toggles": { key: on } } })
}

pub fn weight_patch(key: &str, value: f64) -> Value {
    json!({ "weights": { key: value } })
}

pub fn quality_patch(qn: u32) -> Value {
    json!({ "preferred_quality": qn })
}

/// A list is replaced whole.
pub fn keywords_patch(keywords: &[Value]) -> Value {
    json!({ "rules": { "keywords": keywords } })
}

/// A keyword matched literally against title, tags and description.
pub fn keyword(pattern: &str) -> Value {
    json!({ "pattern": pattern, "regex": false, "fields": ["title", "tags", "desc"] })
}

/// `text` for use inside a XAML attribute.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefs_survive_a_round_trip() {
        let danmaku = Look { area: 1.0, speed: 1.5, size: 26.0, opacity: 0.6, top: false, ..Look::default() };
        let live_filter = LiveFilter { gifts: false, popups: true, effects: false };
        let prefs = Prefs { words: true, pause_on_blur: false, shortcuts: true, volume: 35, danmaku, live_filter, single_column: false };
        assert_eq!(Prefs::parse(&prefs.to_text()), prefs);
        assert_eq!(Prefs::parse("live_volume=250").volume, 100);
        assert_eq!(Prefs::parse("live_volume=loud").volume, DEFAULT_VOLUME);
        assert_eq!(Prefs::parse("words=1"), Prefs { words: true, ..Prefs::default() });
        assert_eq!(Prefs::parse("nonsense words"), Prefs::default());
        assert_eq!(Prefs::parse(""), Prefs::default());
    }

    #[test]
    fn patches_touch_only_what_changed() {
        assert_eq!(rule_patch("ad", false).to_string(), r#"{"rules":{"toggles":{"ad":false}}}"#);
        assert_eq!(weight_patch("topic", 0.5).to_string(), r#"{"weights":{"topic":0.5}}"#);
        assert_eq!(quality_patch(64).to_string(), r#"{"preferred_quality":64}"#);
        let list = keywords_patch(&[keyword("抽奖")]);
        assert_eq!(list["rules"]["keywords"][0]["pattern"], "抽奖");
    }

    #[test]
    fn every_patch_is_one_the_core_understands() {
        let settings = serde_json::to_value(rubi_core::Settings::default()).unwrap();
        for rule in FILTERS.iter().chain(&DANMAKU) {
            assert!(settings["rules"]["toggles"][rule.key].is_boolean(), "{}", rule.key);
        }
        for (key, _) in FACTORS {
            assert!(settings["weights"][key].is_number(), "{key}");
        }
        assert!(settings["preferred_quality"].is_number());
    }

    #[test]
    fn text_cannot_break_out_of_an_attribute() {
        assert_eq!(escape(r#"a<"&">"#), "a&lt;&quot;&amp;&quot;&gt;");
    }
}
