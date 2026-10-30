//! Filtering: a composable rule pipeline for feed cards, a configurable rule table,
//! and danmaku processing (command-danmaku removal, 空降 merging, track layout).

pub mod danmaku;
mod field;
mod pipeline;
mod ruleset;
mod video_rules;

pub use field::{Condition, FieldRule, HasRaw, Target};
pub use pipeline::{Action, Category, Decision, Hit, Pipeline, Rule, Verdict};
pub use ruleset::{DurationRange, KeywordField, KeywordSpec, RuleSet, RuleSetError, Toggles};
pub use video_rules::{
    DurationRule, KeywordRule, NonVideoRule, PubdateRule, UpBlacklistRule, WatchedLookup, WatchedRule, ZoneBlockRule,
};
