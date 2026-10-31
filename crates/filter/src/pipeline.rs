use serde::{Deserialize, Serialize};

/// What kind of thing a rule removes or demotes. Each category has a user-facing toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// 必火推广
    Promotion,
    Ad,
    Banner,
    Goods,
    PinnedComment,
    /// 商业合作稿件 (demoted, never blocked)
    Commercial,
    NonVideo,
    Keyword,
    UpBlock,
    Zone,
    Duration,
    Pubdate,
    Watched,
    CommandDanmaku,
    AdvancedDanmaku,
    JumpMerge,
    DanmakuKeyword,
    DanmakuUser,
}

impl Category {
    /// The name users see (filter summaries, the 被过滤内容 page).
    pub fn label(self) -> &'static str {
        match self {
            Category::Promotion => "必火推广",
            Category::Ad => "广告",
            Category::Banner => "横幅",
            Category::Goods => "商品卡片",
            Category::PinnedComment => "置顶带货评论",
            Category::Commercial => "商业合作",
            Category::NonVideo => "非视频内容",
            Category::Keyword => "屏蔽词",
            Category::UpBlock => "屏蔽 UP",
            Category::Zone => "屏蔽分区",
            Category::Duration => "时长",
            Category::Pubdate => "发布时间",
            Category::Watched => "已看过",
            Category::CommandDanmaku => "互动弹幕",
            Category::AdvancedDanmaku => "高级弹幕",
            Category::JumpMerge => "空降合并",
            Category::DanmakuKeyword => "弹幕屏蔽词",
            Category::DanmakuUser => "屏蔽弹幕用户",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Block,
    /// Multiplies the item's standing by `factor` (0..1) instead of removing it.
    Penalize {
        factor: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Pass,
    Block { reason: String },
    Penalize { factor: f32, reason: String },
}

/// One rule's judgement on one item.
pub trait Rule<T: ?Sized>: Send + Sync {
    fn id(&self) -> &str;
    fn category(&self) -> Category;
    fn judge(&self, item: &T) -> Verdict;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub rule_id: String,
    pub category: Category,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub blocked_by: Option<Hit>,
    pub penalties: Vec<Hit>,
    /// Product of all penalty factors; 1.0 when nothing applied.
    pub multiplier: f32,
}

impl Decision {
    pub fn is_blocked(&self) -> bool {
        self.blocked_by.is_some()
    }
}

/// An ordered set of rules. The first blocking rule wins; penalties accumulate.
pub struct Pipeline<T: ?Sized> {
    rules: Vec<Box<dyn Rule<T>>>,
}

impl<T: ?Sized> Default for Pipeline<T> {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

impl<T: ?Sized> Pipeline<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, rule: impl Rule<T> + 'static) -> Self {
        self.rules.push(Box::new(rule));
        self
    }

    pub fn push(&mut self, rule: Box<dyn Rule<T>>) {
        self.rules.push(rule);
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub fn judge(&self, item: &T) -> Decision {
        let mut penalties = Vec::new();
        let mut multiplier = 1.0;
        for rule in &self.rules {
            let hit = |reason: String| Hit { rule_id: rule.id().to_owned(), category: rule.category(), reason };
            match rule.judge(item) {
                Verdict::Pass => {}
                Verdict::Block { reason } => {
                    return Decision { blocked_by: Some(hit(reason)), penalties, multiplier };
                }
                Verdict::Penalize { factor, reason } => {
                    multiplier *= factor.clamp(0.0, 1.0);
                    penalties.push(hit(reason));
                }
            }
        }
        Decision { blocked_by: None, penalties, multiplier }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(&'static str, Verdict);
    impl Rule<str> for Fixed {
        fn id(&self) -> &str {
            self.0
        }
        fn category(&self) -> Category {
            Category::Keyword
        }
        fn judge(&self, _: &str) -> Verdict {
            self.1.clone()
        }
    }

    #[test]
    fn first_block_wins_and_penalties_multiply() {
        let p = Pipeline::<str>::new()
            .with(Fixed("a", Verdict::Penalize { factor: 0.5, reason: "a".into() }))
            .with(Fixed("b", Verdict::Penalize { factor: 0.5, reason: "b".into() }));
        let d = p.judge("x");
        assert!(!d.is_blocked());
        assert_eq!(d.multiplier, 0.25);

        let p = p
            .with(Fixed("c", Verdict::Block { reason: "c".into() }))
            .with(Fixed("d", Verdict::Block { reason: "d".into() }));
        assert_eq!(p.judge("x").blocked_by.unwrap().rule_id, "c");
    }
}
