//! The rule table applied to comments, hot words and chats; used by the services.

use rubi_api::{CommentApi, CommentArea, CommentPage, CommentSort, HotWord};
use rubi_filter::{RuleSet, Target};
use rubi_model::{ChatSession, Comment};

use crate::error::Result;

/// Replies fetched per page when a thread is expanded.
const THREAD_PAGE: u32 = 20;

#[derive(Debug, Clone, Default)]
pub(crate) struct FilteredComments {
    pub page: CommentPage,
    /// Pinned comments removed by the 置顶带货 rule.
    pub hidden_pinned: usize,
}

/// One page of root comments; pinned 带货 comments are dropped.
pub(crate) async fn load_comments(
    api: &dyn CommentApi,
    rules: &RuleSet,
    area: CommentArea,
    sort: CommentSort,
    offset: Option<&str>,
) -> Result<FilteredComments> {
    let mut page = api.page(area, sort, offset).await?;
    let pipeline = rules.raw_pipeline(Target::Comment);
    let before = page.pinned.len();
    page.pinned.retain(|c| !pipeline.judge(&c.raw).is_blocked());
    Ok(FilteredComments { hidden_pinned: before - page.pinned.len(), page })
}

/// Every reply under `root` (up to `max_pages` pages), flat. Feed it to
/// [`rubi_model::flatten_thread`] to get the fishbone rows.
pub(crate) async fn load_thread(
    api: &dyn CommentApi,
    area: CommentArea,
    root: u64,
    max_pages: u32,
) -> Result<Vec<Comment>> {
    let mut all = Vec::new();
    for pn in 1..=max_pages.max(1) {
        let (replies, total) = api.replies(area, root, pn, THREAD_PAGE).await?;
        let done = replies.is_empty();
        all.extend(replies);
        if done || all.len() as u32 >= total {
            break;
        }
    }
    Ok(all)
}

pub(crate) fn filter_hot_words(rules: &RuleSet, words: Vec<HotWord>) -> (Vec<HotWord>, usize) {
    let pipeline = rules.raw_pipeline(Target::HotWord);
    let before = words.len();
    let kept: Vec<HotWord> = words.into_iter().filter(|w| !pipeline.judge(&w.raw).is_blocked()).collect();
    let hidden = before - kept.len();
    (kept, hidden)
}

pub(crate) fn filter_sessions(rules: &RuleSet, sessions: Vec<ChatSession>) -> (Vec<ChatSession>, usize) {
    let pipeline = rules.raw_pipeline(Target::Notice);
    let before = sessions.len();
    let kept: Vec<ChatSession> = sessions.into_iter().filter(|s| !pipeline.judge(&s.raw).is_blocked()).collect();
    let hidden = before - kept.len();
    (kept, hidden)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;

    struct Fake;

    #[async_trait]
    impl CommentApi for Fake {
        async fn page(&self, _: CommentArea, _: CommentSort, _: Option<&str>) -> rubi_api::Result<CommentPage> {
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

        async fn replies(&self, _: CommentArea, _: u64, pn: u32, _: u32) -> rubi_api::Result<(Vec<Comment>, u32)> {
            let c = Comment { rpid: u64::from(pn), ..Default::default() };
            Ok((vec![c], 2))
        }

        async fn root(&self, _: CommentArea, root: u64) -> rubi_api::Result<Comment> {
            Ok(Comment { rpid: root, ..Default::default() })
        }
        async fn like(&self, _: CommentArea, _: u64, _: bool) -> rubi_api::Result<()> {
            Ok(())
        }
        async fn reply(
            &self,
            _: CommentArea,
            _: u64,
            _: u64,
            _: &str,
            _: &[(String, u64)],
        ) -> rubi_api::Result<Comment> {
            Ok(Comment::default())
        }
    }

    #[tokio::test]
    async fn pinned_goods_comment_is_hidden() {
        let r = load_comments(&Fake, &RuleSet::builtin(), CommentArea::video(1), CommentSort::Hot, None).await.unwrap();
        assert_eq!(r.hidden_pinned, 1);
        assert_eq!(r.page.pinned[0].rpid, 2);
    }

    #[tokio::test]
    async fn thread_stops_at_total() {
        let all = load_thread(&Fake, CommentArea::video(1), 1, 5).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn promo_sessions_and_hot_words_are_hidden() {
        let rules = RuleSet::builtin();
        // Shapes from the 2026-10-31 capture of get_sessions.
        let s = |msg_type, system, follow| ChatSession {
            raw: json!({"last_msg": {"msg_type": msg_type}, "system_msg_type": system, "is_follow": follow}),
            ..Default::default()
        };
        let official = s(10, 7, 1); // 服务通知 / 创作助手
        let followed_card = s(10, 0, 1);
        let promo = s(10, 0, 0);
        let (kept, hidden) = filter_sessions(&rules, vec![s(1, 0, 0), official, followed_card, promo]);
        assert_eq!((kept.len(), hidden), (3, 1));
        let w = |c: &str| HotWord { keyword: "k".into(), raw: json!({"is_commercial": c}) };
        assert_eq!(filter_hot_words(&rules, vec![w("0"), w("1")]).1, 1);
    }
}
