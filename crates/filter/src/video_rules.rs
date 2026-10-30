//! Typed rules that the user configures directly.

use std::collections::HashSet;
use std::sync::Arc;

use regex::Regex;
use rubi_model::VideoCard;

use crate::pipeline::{Category, Rule, Verdict};
use crate::ruleset::KeywordField;

pub struct KeywordRule {
    id: String,
    pattern: Regex,
    fields: Vec<KeywordField>,
}

impl KeywordRule {
    pub fn new(id: impl Into<String>, pattern: Regex, fields: Vec<KeywordField>) -> Self {
        Self { id: id.into(), pattern, fields }
    }
}

impl Rule<VideoCard> for KeywordRule {
    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> Category {
        Category::Keyword
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        let hit = self.fields.iter().any(|f| match f {
            KeywordField::Title => self.pattern.is_match(&item.title),
            KeywordField::Desc => self.pattern.is_match(&item.desc),
            KeywordField::Tags => item.tags.iter().any(|t| self.pattern.is_match(t)),
        });
        if hit { Verdict::Block { reason: format!("关键词 {}", self.pattern.as_str()) } } else { Verdict::Pass }
    }
}

/// Catch-all for cards that are not videos (live rooms, games…) and were not
/// already classified by a more specific field rule.
pub struct NonVideoRule;

impl Rule<VideoCard> for NonVideoRule {
    fn id(&self) -> &str {
        "builtin.non_video"
    }

    fn category(&self) -> Category {
        Category::NonVideo
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        if item.is_video() { Verdict::Pass } else { Verdict::Block { reason: "非视频卡片".into() } }
    }
}

pub struct UpBlacklistRule(pub HashSet<u64>);

impl Rule<VideoCard> for UpBlacklistRule {
    fn id(&self) -> &str {
        "user.up_blacklist"
    }

    fn category(&self) -> Category {
        Category::UpBlock
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        if self.0.contains(&item.owner.mid) {
            Verdict::Block { reason: format!("UP 黑名单 {}", item.owner.name) }
        } else {
            Verdict::Pass
        }
    }
}

pub struct ZoneBlockRule(pub HashSet<u32>);

impl Rule<VideoCard> for ZoneBlockRule {
    fn id(&self) -> &str {
        "user.zone_block"
    }

    fn category(&self) -> Category {
        Category::Zone
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        match &item.zone {
            Some(z) if self.0.contains(&z.tid) => Verdict::Block { reason: format!("屏蔽分区 {}", z.name) },
            _ => Verdict::Pass,
        }
    }
}

pub struct DurationRule {
    pub min_s: Option<u32>,
    pub max_s: Option<u32>,
}

impl Rule<VideoCard> for DurationRule {
    fn id(&self) -> &str {
        "user.duration"
    }

    fn category(&self) -> Category {
        Category::Duration
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        let d = item.duration_s;
        // Unknown durations (0) are never filtered.
        let out = d > 0 && (self.min_s.is_some_and(|m| d < m) || self.max_s.is_some_and(|m| d > m));
        if out { Verdict::Block { reason: format!("时长 {d}s 超出范围") } } else { Verdict::Pass }
    }
}

/// Blocks videos older than `max_age_days`, measured against a clock the caller provides.
pub struct PubdateRule {
    pub max_age_days: u32,
    pub now: fn() -> i64,
}

impl Rule<VideoCard> for PubdateRule {
    fn id(&self) -> &str {
        "user.pubdate"
    }

    fn category(&self) -> Category {
        Category::Pubdate
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        let age = (self.now)() - item.pubdate;
        if item.pubdate > 0 && age > i64::from(self.max_age_days) * 86_400 {
            Verdict::Block { reason: format!("发布超过 {} 天", self.max_age_days) }
        } else {
            Verdict::Pass
        }
    }
}

/// Answers whether the user has already watched a video. Implemented by the store.
pub trait WatchedLookup: Send + Sync {
    fn is_watched(&self, bvid: &str) -> bool;
}

pub struct WatchedRule(pub Arc<dyn WatchedLookup>);

impl Rule<VideoCard> for WatchedRule {
    fn id(&self) -> &str {
        "builtin.watched"
    }

    fn category(&self) -> Category {
        Category::Watched
    }

    fn judge(&self, item: &VideoCard) -> Verdict {
        if item.is_video() && self.0.is_watched(&item.bvid) {
            Verdict::Block { reason: "已看过".into() }
        } else {
            Verdict::Pass
        }
    }
}
