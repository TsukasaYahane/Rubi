//! Rules that match fields of the raw API JSON. These are data, not code, so a changed
//! API field only needs a rule-table update.

use rubi_model::VideoCard;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::pipeline::{Action, Category, Rule, Verdict};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Condition {
    Eq {
        path: String,
        value: Value,
    },
    In {
        path: String,
        values: Vec<Value>,
    },
    /// Present and not null / empty.
    Exists {
        path: String,
    },
    Missing {
        path: String,
    },
    Contains {
        path: String,
        text: String,
    },
    All {
        of: Vec<Condition>,
    },
    Any {
        of: Vec<Condition>,
    },
    Not {
        cond: Box<Condition>,
    },
}

impl Condition {
    /// A path with `*` segments matches when any of the values it reaches does.
    pub fn matches(&self, raw: &Value) -> bool {
        match self {
            Condition::Eq { path, value } => lookup(raw, path).iter().any(|v| loosely_equal(v, value)),
            Condition::In { path, values } => {
                lookup(raw, path).iter().any(|v| values.iter().any(|x| loosely_equal(v, x)))
            }
            Condition::Exists { path } => lookup(raw, path).into_iter().any(is_present),
            Condition::Missing { path } => !lookup(raw, path).into_iter().any(is_present),
            Condition::Contains { path, text } => {
                lookup(raw, path).into_iter().filter_map(Value::as_str).any(|s| s.contains(text.as_str()))
            }
            Condition::All { of } => of.iter().all(|c| c.matches(raw)),
            Condition::Any { of } => of.iter().any(|c| c.matches(raw)),
            Condition::Not { cond } => !cond.matches(raw),
        }
    }
}

/// Dotted path lookup; numeric segments index arrays (`a.0.b`) and `*` takes every value of
/// an object or array (`content.jump_url.*.extra`), so one path can reach several values.
fn lookup<'a>(v: &'a Value, path: &str) -> Vec<&'a Value> {
    path.split('.').fold(vec![v], |found, seg| {
        found
            .into_iter()
            .flat_map(|cur| -> Vec<&Value> {
                match (cur, seg) {
                    (Value::Object(m), "*") => m.values().collect(),
                    (Value::Array(a), "*") => a.iter().collect(),
                    (Value::Object(m), _) => m.get(seg).into_iter().collect(),
                    (Value::Array(a), _) => seg.parse::<usize>().ok().and_then(|i| a.get(i)).into_iter().collect(),
                    _ => Vec::new(),
                }
            })
            .collect()
    })
}

fn is_present(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::Bool(b) => *b,
    }
}

/// Numbers compare by value and `"1"` equals `1`, because the API mixes both.
fn loosely_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::String(s), Value::Number(n)) | (Value::Number(n), Value::String(s)) => {
            s.parse::<f64>().ok() == n.as_f64()
        }
        _ => a == b,
    }
}

/// Anything that carries the raw API JSON it was parsed from.
pub trait HasRaw {
    fn raw(&self) -> &Value;
}

impl HasRaw for VideoCard {
    fn raw(&self) -> &Value {
        &self.raw
    }
}

impl HasRaw for Value {
    fn raw(&self) -> &Value {
        self
    }
}

/// Which kind of object a field rule applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// Feed / list cards.
    #[default]
    Card,
    /// Home-page banners and event pages.
    Banner,
    /// Goods cards on the video page.
    Goods,
    /// Pinned comments.
    Comment,
    /// Message-center notices and private-message sessions.
    Notice,
    /// Trending search words.
    HotWord,
    /// Paragraphs of a 专栏 / 图文 body (promotion cards, shop links).
    ArticleBlock,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldRule {
    pub id: String,
    pub name: String,
    pub category: Category,
    #[serde(default)]
    pub target: Target,
    #[serde(default = "block")]
    pub action: Action,
    pub when: Condition,
    /// False until the field has been confirmed by packet capture.
    #[serde(default)]
    pub verified: bool,
}

fn block() -> Action {
    Action::Block
}

impl<T: HasRaw> Rule<T> for FieldRule {
    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> Category {
        self.category
    }

    fn judge(&self, item: &T) -> Verdict {
        if !self.when.matches(item.raw()) {
            return Verdict::Pass;
        }
        match self.action {
            Action::Block => Verdict::Block { reason: self.name.clone() },
            Action::Penalize { factor } => Verdict::Penalize { factor, reason: self.name.clone() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn conditions_match_raw_json() {
        let raw = json!({"goto": "ad", "business_info": {"id": 7}, "list": [{"x": "1"}], "empty": {}});
        assert!(Condition::Eq { path: "goto".into(), value: json!("ad") }.matches(&raw));
        assert!(Condition::In { path: "goto".into(), values: vec![json!("av"), json!("ad")] }.matches(&raw));
        assert!(Condition::Exists { path: "business_info".into() }.matches(&raw));
        assert!(Condition::Missing { path: "empty".into() }.matches(&raw));
        assert!(Condition::Missing { path: "nope.deeper".into() }.matches(&raw));
        assert!(Condition::Eq { path: "list.0.x".into(), value: json!(1) }.matches(&raw));
        assert!(Condition::Not { cond: Box::new(Condition::Exists { path: "nope".into() }) }.matches(&raw));
    }

    #[test]
    fn wildcard_segments_match_any_value() {
        let raw = json!({"m": {"k1": {"x": 0}, "k2": {"x": 5}}, "a": [{"t": "淘宝"}, {"t": "b"}]});
        assert!(Condition::Eq { path: "m.*.x".into(), value: json!(5) }.matches(&raw));
        assert!(Condition::Exists { path: "m.*.x".into() }.matches(&raw));
        assert!(Condition::Contains { path: "a.*.t".into(), text: "淘".into() }.matches(&raw));
        assert!(Condition::Missing { path: "m.*.y".into() }.matches(&raw));
        assert!(!Condition::Eq { path: "m.*.x".into(), value: json!(7) }.matches(&raw));
    }
}
