//! The user-editable rule table: built-in toggles plus custom rules, stored as TOML.

use std::collections::HashSet;
use std::sync::Arc;

use regex::{Regex, RegexBuilder};
use rubi_model::VideoCard;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::danmaku::DanmakuOptions;
use crate::field::{FieldRule, Target};
use crate::pipeline::{Category, Pipeline};
use crate::video_rules::{
    DurationRule, KeywordRule, NonVideoRule, PubdateRule, UpBlacklistRule, WatchedLookup, WatchedRule, ZoneBlockRule,
};

const BUILTIN: &str = include_str!("default_rules.toml");

#[derive(Debug, thiserror::Error)]
pub enum RuleSetError {
    #[error("rule table is not valid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("rule table could not be written: {0}")]
    Write(#[from] toml::ser::Error),
    #[error("keyword {pattern:?} is not a valid regex: {source}")]
    Regex { pattern: String, source: regex::Error },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Toggles {
    pub promotion: bool,
    pub ad: bool,
    pub banner: bool,
    pub goods: bool,
    pub pinned_comment: bool,
    pub commercial_penalty: bool,
    pub non_video: bool,
    pub watched: bool,
    pub command_danmaku: bool,
    pub show_advanced_danmaku: bool,
    pub merge_jump: bool,
    pub jump_marks: bool,
    pub notice: bool,
}

impl Default for Toggles {
    fn default() -> Self {
        Self {
            promotion: true,
            ad: true,
            banner: true,
            goods: true,
            pinned_comment: true,
            commercial_penalty: false,
            non_video: true,
            watched: true,
            command_danmaku: true,
            show_advanced_danmaku: true,
            merge_jump: true,
            jump_marks: true,
            notice: true,
        }
    }
}

impl Toggles {
    /// Whether rules of `category` are active. User-written rules are always active.
    pub fn allows(&self, category: Category) -> bool {
        match category {
            Category::Promotion => self.promotion,
            Category::Ad => self.ad,
            Category::Banner => self.banner,
            Category::Goods => self.goods,
            Category::PinnedComment => self.pinned_comment,
            Category::Commercial => self.commercial_penalty,
            Category::NonVideo => self.non_video,
            Category::Watched => self.watched,
            Category::CommandDanmaku => self.command_danmaku,
            Category::AdvancedDanmaku => !self.show_advanced_danmaku,
            Category::JumpMerge => self.merge_jump,
            Category::Keyword
            | Category::UpBlock
            | Category::Zone
            | Category::Duration
            | Category::Pubdate
            | Category::DanmakuKeyword
            | Category::DanmakuUser => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeywordField {
    Title,
    Tags,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeywordSpec {
    pub pattern: String,
    /// When false the pattern is matched literally.
    #[serde(default)]
    pub regex: bool,
    #[serde(default = "all_fields")]
    pub fields: Vec<KeywordField>,
}

fn all_fields() -> Vec<KeywordField> {
    vec![KeywordField::Title, KeywordField::Tags, KeywordField::Desc]
}

impl KeywordSpec {
    pub fn compile(&self) -> Result<Regex, RuleSetError> {
        let src = if self.regex { self.pattern.clone() } else { regex::escape(&self.pattern) };
        RegexBuilder::new(&src)
            .case_insensitive(true)
            .build()
            .map_err(|source| RuleSetError::Regex { pattern: self.pattern.clone(), source })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DurationRange {
    pub min_s: Option<u32>,
    pub max_s: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuleSet {
    pub version: u32,
    /// Where the table came from; reserved for remote subscriptions.
    pub source: String,
    pub toggles: Toggles,
    pub field_rules: Vec<FieldRule>,
    pub keywords: Vec<KeywordSpec>,
    pub up_blacklist: Vec<u64>,
    pub zone_block: Vec<u32>,
    pub duration: Option<DurationRange>,
    pub max_age_days: Option<u32>,
    pub danmaku_keywords: Vec<KeywordSpec>,
    pub danmaku_blocked_users: Vec<String>,
}

/// An empty table with default toggles. Use [`RuleSet::builtin`] for the shipped rules;
/// this must not parse the built-in TOML, since serde's field defaults call it.
impl Default for RuleSet {
    fn default() -> Self {
        Self {
            version: 1,
            source: "user".into(),
            toggles: Toggles::default(),
            field_rules: Vec::new(),
            keywords: Vec::new(),
            up_blacklist: Vec::new(),
            zone_block: Vec::new(),
            duration: None,
            max_age_days: None,
            danmaku_keywords: Vec::new(),
            danmaku_blocked_users: Vec::new(),
        }
    }
}

impl RuleSet {
    pub fn builtin() -> Self {
        toml::from_str(BUILTIN).expect("built-in rule table is valid")
    }

    pub fn from_toml(text: &str) -> Result<Self, RuleSetError> {
        Ok(toml::from_str(text)?)
    }

    pub fn to_toml(&self) -> Result<String, RuleSetError> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Adds built-in field rules this table does not have yet (matched by id) and
    /// replaces older built-in ones. User-added rules are left alone.
    pub fn merge_builtin(&mut self) {
        let builtin = Self::builtin();
        for rule in builtin.field_rules {
            match self.field_rules.iter_mut().find(|r| r.id == rule.id) {
                Some(existing) => *existing = rule,
                None => self.field_rules.push(rule),
            }
        }
        self.version = self.version.max(builtin.version);
    }

    /// The pipeline for feed and list cards.
    pub fn card_pipeline(
        &self,
        watched: Option<Arc<dyn WatchedLookup>>,
        now: fn() -> i64,
    ) -> Result<Pipeline<VideoCard>, RuleSetError> {
        let mut p = Pipeline::new();
        for rule in self.active_field_rules(Target::Card) {
            p.push(Box::new(rule));
        }
        if self.toggles.non_video {
            p.push(Box::new(NonVideoRule));
        }
        if !self.up_blacklist.is_empty() {
            p.push(Box::new(UpBlacklistRule(self.up_blacklist.iter().copied().collect())));
        }
        if !self.zone_block.is_empty() {
            p.push(Box::new(ZoneBlockRule(self.zone_block.iter().copied().collect())));
        }
        if let Some(d) = &self.duration {
            p.push(Box::new(DurationRule { min_s: d.min_s, max_s: d.max_s }));
        }
        if let Some(days) = self.max_age_days {
            p.push(Box::new(PubdateRule { max_age_days: days, now }));
        }
        for (i, k) in self.keywords.iter().enumerate() {
            p.push(Box::new(KeywordRule::new(format!("user.keyword.{i}"), k.compile()?, k.fields.clone())));
        }
        if let (true, Some(w)) = (self.toggles.watched, watched) {
            p.push(Box::new(WatchedRule(w)));
        }
        Ok(p)
    }

    /// A pipeline over raw JSON for banners, goods cards, pinned comments or notices.
    pub fn raw_pipeline(&self, target: Target) -> Pipeline<Value> {
        let mut p = Pipeline::new();
        for rule in self.active_field_rules(target) {
            p.push(Box::new(rule));
        }
        p
    }

    pub fn danmaku_options(&self) -> Result<DanmakuOptions, RuleSetError> {
        Ok(DanmakuOptions {
            drop_commands: self.toggles.command_danmaku,
            show_advanced: self.toggles.show_advanced_danmaku,
            merge_jump: self.toggles.merge_jump,
            keywords: self.danmaku_keywords.iter().map(KeywordSpec::compile).collect::<Result<_, _>>()?,
            blocked_users: self.danmaku_blocked_users.iter().cloned().collect::<HashSet<_>>(),
        })
    }

    fn active_field_rules(&self, target: Target) -> impl Iterator<Item = FieldRule> + '_ {
        self.field_rules.iter().filter(move |r| r.target == target && self.toggles.allows(r.category)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn card(raw: Value, bvid: &str) -> VideoCard {
        VideoCard { bvid: bvid.into(), raw, ..Default::default() }
    }

    #[test]
    fn bihuo_marks_on_feed_and_search_cards_are_blocked() {
        let rs = RuleSet::builtin();
        let p = rs.card_pipeline(None, || 0).unwrap();
        let card = |raw: Value| VideoCard { bvid: "BV1".into(), raw, ..Default::default() };
        assert!(p.judge(&card(json!({"rcmd_reason": {"content": "必火推荐"}}))).is_blocked());
        assert!(p.judge(&card(json!({"corner": "必火推广"}))).is_blocked());
        assert!(!p.judge(&card(json!({"corner": "", "rec_reason": "", "spread_id": 0}))).is_blocked());
    }

    /// 2026-11-03, www.bilibili.com 首页: a 必火 card (rocket on the cover) is an ad slot carrying a
    /// real video; it is told apart from a plain ad and logged as 必火推广.
    #[test]
    fn bihuo_rocket_cards_are_promotion() {
        let p = RuleSet::builtin().card_pipeline(None, || 0).unwrap();
        let card = |raw: Value| VideoCard { bvid: "BV1".into(), raw, ..Default::default() };
        let rocket = json!({"goto": "ad", "business_info": {"is_ad": true, "card_type": 82, "creative_type": 1,
            "archive": {"aid": 1, "title": "戴它，不是为了好看"},
            "business_mark": {"type": 4, "img_url": "https://i0.hdslb.com/bfs/sycp/mng/202203/3621990414ef2de04c5730f1002d8dae.png"}}});
        let d = p.judge(&card(rocket));
        assert_eq!(d.blocked_by.map(|h| h.rule_id).as_deref(), Some("promotion.bihuo_card"));
        let plain_ad = json!({"goto": "ad", "business_info": {"is_ad": true, "card_type": 0, "creative_type": 2,
            "business_mark": {"type": 2, "text": "广告"}}});
        assert_eq!(p.judge(&card(plain_ad)).blocked_by.map(|h| h.rule_id).as_deref(), Some("ad.goto"));
    }

    /// 2026-11-01: an UP's pinned Taobao links (b23.tv/mall-…, goods_item_id, no prefetch cache).
    #[test]
    fn pinned_mall_links_are_goods() {
        let p = RuleSet::builtin().raw_pipeline(Target::Comment);
        let pinned = json!({"content": {"message": "推荐 https://b23.tv/mall-1PEUvH-75mBT",
            "jump_url": {"https://b23.tv/mall-1PEUvH-75mBT": {"extra": {"goods_item_id": 104811215, "is_word_search": false}}}}});
        assert!(p.judge(&pinned).is_blocked());
        let highlight =
            json!({"content": {"message": "看看 话题", "jump_url": {"话题": {"extra": {"is_word_search": true}}}}});
        assert!(!p.judge(&highlight).is_blocked());
    }

    #[test]
    fn article_promo_cards_are_blocked() {
        let p = RuleSet::builtin().raw_pipeline(Target::ArticleBlock);
        assert!(
            p.judge(&json!({"para_type": 6, "link_card": {"card": {"type": "LINK_CARD_TYPE_GOODS"}}})).is_blocked()
        );
        let shop = json!({"para_type": 1, "text": {"nodes": [{"rich": {"jump_url": "https://b23.tv/cm-abc"}}]}});
        assert!(p.judge(&shop).is_blocked());
        let video = json!({"para_type": 6, "link_card": {"card": {"type": "LINK_CARD_TYPE_UGC"}}});
        assert!(!p.judge(&video).is_blocked());
        assert!(!p.judge(&json!({"para_type": 1, "text": {"nodes": [{"word": {"words": "正文"}}]}})).is_blocked());
    }

    #[test]
    fn builtin_parses_and_round_trips() {
        let rs = RuleSet::builtin();
        assert!(rs.field_rules.len() >= 6);
        let back = RuleSet::from_toml(&rs.to_toml().unwrap()).unwrap();
        assert_eq!(back, rs);
    }

    #[test]
    fn ad_cards_are_blocked_and_videos_pass() {
        let p = RuleSet::builtin().card_pipeline(None, || 0).unwrap();
        let ad = p.judge(&card(json!({"goto": "ad", "business_info": {"id": 1}}), ""));
        assert_eq!(ad.blocked_by.unwrap().category, Category::Ad);
        assert!(!p.judge(&card(json!({"goto": "av"}), "BV1xx")).is_blocked());
        let live = p.judge(&card(json!({"goto": "live"}), ""));
        assert_eq!(live.blocked_by.unwrap().category, Category::Promotion);
    }

    /// Shapes from the 2026-10-31 capture of the web home feed (x/web-interface/index/top/feed/rcmd).
    #[test]
    fn captured_feed_cards() {
        let p = RuleSet::builtin().card_pipeline(None, || 0).unwrap();
        let ad = json!({"goto": "ad", "id": 0, "bvid": "", "rcmd_reason": null, "business_info": {
            "is_ad": true, "cm_mark": 1, "creative_type": 2, "business_mark": {"text": "广告", "type": 2}}});
        let hit = p.judge(&card(ad, "")).blocked_by.unwrap();
        assert_eq!((hit.rule_id.as_str(), hit.category), ("ad.goto", Category::Ad));
        let video = json!({"goto": "av", "business_info": null, "room_info": null, "ogv_info": null,
            "av_feature": null, "rcmd_reason": {"reason_type": 0}});
        assert!(!p.judge(&card(video, "BV1kRna67Eza")).is_blocked());
    }

    /// Pinned comments from the 2026-10-31 capture of x/v2/reply/main (ranking videos).
    #[test]
    fn captured_pinned_comments() {
        let p = RuleSet::builtin().raw_pipeline(Target::Comment);
        let goods = json!({"content": {"message": "https://b23.tv/cm-yaoyue", "jump_url": {
            "https://b23.tv/cm-yaoyue": {"title": "爱回收严选D100｜真低价，不怕比！", "pc_url": "https://m.aihuishou.com/activity/x",
                "extra": {"goods_prefetched_cache": "{\"request_id\":\"1\"}"}}}}});
        assert!(p.judge(&goods).is_blocked());
        let word = json!({"content": {"message": "本视频由updream制作", "jump_url": {"updream": {"title": "updream",
            "pc_url": "//search.bilibili.com/all?keyword=updream",
            "extra": {"goods_show_type": 0, "is_word_search": true, "goods_cm_control": 0}}}}});
        assert!(!p.judge(&word).is_blocked());
        let bangumi = json!({"content": {"message": "来都来了，那就看看正片吧~", "jump_url": {
            "https://www.bilibili.com/bangumi/play/ss46632": {"title": "西游记", "pc_url": ""}}}});
        assert!(!p.judge(&bangumi).is_blocked());
    }

    /// Shape from the 2026-10-31 capture of s.search.bilibili.com/main/hotword.
    #[test]
    fn captured_hot_words() {
        let p = RuleSet::builtin().raw_pipeline(Target::HotWord);
        let plain = json!({"keyword": "韩立", "word_type": 6, "goto_type": 0, "stat_datas": {"is_commercial": "0"}});
        assert!(!p.judge(&plain).is_blocked());
        let paid = json!({"keyword": "x", "stat_datas": {"is_commercial": "1"}});
        assert!(p.judge(&paid).is_blocked());
    }

    #[test]
    fn toggles_switch_rules_off() {
        let mut rs = RuleSet::builtin();
        rs.toggles.ad = false;
        rs.toggles.non_video = false;
        let p = rs.card_pipeline(None, || 0).unwrap();
        assert!(!p.judge(&card(json!({"goto": "ad"}), "")).is_blocked());
    }

    #[test]
    fn commercial_is_penalized_only_when_enabled() {
        let raw = json!({"goto": "av", "is_commercial": 1});
        let off = RuleSet::builtin().card_pipeline(None, || 0).unwrap().judge(&card(raw.clone(), "BV1"));
        assert_eq!(off.multiplier, 1.0);
        let mut rs = RuleSet::builtin();
        rs.toggles.commercial_penalty = true;
        let on = rs.card_pipeline(None, || 0).unwrap().judge(&card(raw, "BV1"));
        assert!(!on.is_blocked());
        assert!((on.multiplier - 0.85).abs() < 1e-6);
    }

    #[test]
    fn user_rules_apply() {
        let mut rs = RuleSet::builtin();
        rs.keywords.push(KeywordSpec { pattern: "剧透".into(), regex: false, fields: all_fields() });
        rs.up_blacklist.push(42);
        let p = rs.card_pipeline(None, || 0).unwrap();
        assert!(
            p.judge(&VideoCard { bvid: "BV1".into(), title: "大结局剧透".into(), ..Default::default() }).is_blocked()
        );
        let mut c = card(json!({}), "BV2");
        c.owner.mid = 42;
        assert_eq!(p.judge(&c).blocked_by.unwrap().category, Category::UpBlock);
    }

    #[test]
    fn banner_and_goods_are_blocked_wholesale() {
        let rs = RuleSet::builtin();
        assert!(rs.raw_pipeline(Target::Banner).judge(&json!({"id": 1})).is_blocked());
        assert!(rs.raw_pipeline(Target::Goods).judge(&json!({})).is_blocked());
        let pinned = rs.raw_pipeline(Target::Comment);
        assert!(pinned.judge(&json!({"content": {"message": "京东自营链接"}})).is_blocked());
        assert!(!pinned.judge(&json!({"content": {"message": "感谢观看"}})).is_blocked());
    }
}
