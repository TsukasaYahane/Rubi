//! Comments, search, message center and chat, with the rule table applied.

use rubi_api::{CommentApi, CommentPage, CommentSort, HotWord};
use rubi_filter::{RuleSet, Target};
use rubi_model::{ChatSession, Comment};

use crate::error::Result;

/// Replies fetched per page when a thread is expanded.
const THREAD_PAGE: u32 = 20;

#[derive(Debug, Clone, Default)]
pub struct FilteredComments {
    pub page: CommentPage,
    /// Pinned comments removed by the 置顶带货 rule.
    pub hidden_pinned: usize,
}

/// One page of root comments; pinned 带货 comments are dropped.
pub async fn load_comments(
    api: &dyn CommentApi,
    rules: &RuleSet,
    aid: u64,
    sort: CommentSort,
    offset: Option<&str>,
) -> Result<FilteredComments> {
    let mut page = api.page(aid, sort, offset).await?;
    let pipeline = rules.raw_pipeline(Target::Comment);
    let before = page.pinned.len();
    page.pinned.retain(|c| !pipeline.judge(&c.raw).is_blocked());
    Ok(FilteredComments { hidden_pinned: before - page.pinned.len(), page })
}

/// Every reply under `root` (up to `max_pages` pages), flat. Feed it to
/// [`rubi_model::flatten_thread`] to get the fishbone rows.
pub async fn load_thread(api: &dyn CommentApi, aid: u64, root: u64, max_pages: u32) -> Result<Vec<Comment>> {
    let mut all = Vec::new();
    for pn in 1..=max_pages.max(1) {
        let (replies, total) = api.replies(aid, root, pn, THREAD_PAGE).await?;
        let done = replies.is_empty();
        all.extend(replies);
        if done || all.len() as u32 >= total {
            break;
        }
    }
    Ok(all)
}

pub fn filter_hot_words(rules: &RuleSet, words: Vec<HotWord>) -> (Vec<HotWord>, usize) {
    let pipeline = rules.raw_pipeline(Target::HotWord);
    let before = words.len();
    let kept: Vec<HotWord> = words.into_iter().filter(|w| !pipeline.judge(&w.raw).is_blocked()).collect();
    let hidden = before - kept.len();
    (kept, hidden)
}

pub fn filter_sessions(rules: &RuleSet, sessions: Vec<ChatSession>) -> (Vec<ChatSession>, usize) {
    let pipeline = rules.raw_pipeline(Target::Notice);
    let before = sessions.len();
    let kept: Vec<ChatSession> = sessions.into_iter().filter(|s| !pipeline.judge(&s.raw).is_blocked()).collect();
    let hidden = before - kept.len();
    (kept, hidden)
}

/// Whether a comment would be hidden by the pinned-comment rules (for previews).
pub fn is_blocked_pinned(rules: &RuleSet, c: &Comment) -> bool {
    rules.raw_pipeline(Target::Comment).judge(&c.raw).is_blocked()
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;

    struct Fake;

    #[async_trait]
    impl CommentApi for Fake {
        async fn page(&self, _: u64, _: CommentSort, _: Option<&str>) -> rubi_api::Result<CommentPage> {
            let pinned = |rpid, msg: &str| Comment {
                rpid,
                pinned: true,
                raw: json!({"content": {"message": msg}}),
                ..Default::default()
            };
            Ok(CommentPage {
                pinned: vec![pinned(1, "淘宝同款链接"), pinned(2, "感谢观看")], ..Default::default()
            })
        }

        async fn replies(&self, _: u64, _: u64, pn: u32, _: u32) -> rubi_api::Result<(Vec<Comment>, u32)> {
            let c = Comment { rpid: u64::from(pn), ..Default::default() };
            Ok((vec![c], 2))
        }

        async fn root(&self, _: u64, root: u64) -> rubi_api::Result<Comment> {
            Ok(Comment { rpid: root, ..Default::default() })
        }
    }

    #[tokio::test]
    async fn pinned_goods_comment_is_hidden() {
        let r = load_comments(&Fake, &RuleSet::builtin(), 1, CommentSort::Hot, None).await.unwrap();
        assert_eq!(r.hidden_pinned, 1);
        assert_eq!(r.page.pinned[0].rpid, 2);
    }

    #[tokio::test]
    async fn thread_stops_at_total() {
        let all = load_thread(&Fake, 1, 1, 5).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn promo_sessions_and_hot_words_are_hidden() {
        let rules = RuleSet::builtin();
        let s = |msg_type| ChatSession { raw: json!({"last_msg": {"msg_type": msg_type}}), ..Default::default() };
        let (kept, hidden) = filter_sessions(&rules, vec![s(1), s(10)]);
        assert_eq!((kept.len(), hidden), (1, 1));
        let w = |c: &str| HotWord { keyword: "k".into(), raw: json!({"is_commercial": c}) };
        assert_eq!(filter_hot_words(&rules, vec![w("0"), w("1")]).1, 1);
    }
}
