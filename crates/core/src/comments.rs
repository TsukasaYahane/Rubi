//! Comments of a video or an article: pages of threads (pinned 带货 removed), expanding a thread, and finding
//! the thread a notice points to. Threads come out as fishbone rows ready to draw.

use std::sync::Arc;

use rubi_api::{CommentApi, CommentArea, CommentSort};
use rubi_model::{Comment, ThreadRow, flatten_thread};

use crate::error::{CoreError, Result};
use crate::ports::RuleSource;
use crate::recent::Recent;
use crate::social::{load_comments, load_thread};

/// Reply pages loaded when a thread is expanded.
const THREAD_MAX_PAGES: u32 = 5;
/// Root comments kept so their threads can be expanded later.
const ROOT_CACHE: usize = 1_000;

/// A thread as rows: each row with its comment, root first.
#[derive(Debug, Clone, Default)]
pub struct ThreadView {
    pub root_rpid: u64,
    pub rows: Vec<(ThreadRow, Comment)>,
    /// Replies not among the rows (not loaded, or past the limit).
    pub hidden: u32,
    pub total_replies: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CommentsPage {
    pub pinned: Vec<Comment>,
    pub threads: Vec<ThreadView>,
    /// Pinned comments removed by the 置顶带货 rule.
    pub hidden_pinned: usize,
    pub next_offset: Option<String>,
}

pub struct Comments {
    api: Arc<dyn CommentApi>,
    rules: Arc<dyn RuleSource>,
    roots: Recent<u64, Comment>,
}

impl Comments {
    pub(crate) fn new(api: Arc<dyn CommentApi>, rules: Arc<dyn RuleSource>) -> Self {
        Self { api, rules, roots: Recent::new(ROOT_CACHE) }
    }

    /// Likes a comment (or takes the like back).
    pub async fn like(&self, area: CommentArea, rpid: u64, on: bool) -> Result<()> {
        Ok(self.api.like(area, rpid, on).await?)
    }

    /// Replies in a thread (`root` 0: a new comment under the video). The text is trimmed and an
    /// empty one refused. Returns the reply as a row to show at once.
    /// Posts `text`. `mentions` are the people picked from the @ list as (name, mid); only those
    /// whose `@name` is still in the text are sent.
    pub async fn reply(
        &self,
        area: CommentArea,
        root: u64,
        parent: u64,
        text: &str,
        mentions: &[(String, u64)],
    ) -> Result<(ThreadRow, Comment)> {
        let text = text.trim();
        if text.is_empty() {
            return Err(CoreError::Invalid("评论不能为空".into()));
        }
        let at = mentioned(text, mentions);
        let c = self.api.reply(area, root, parent, text, &at).await?;
        let row = ThreadRow { rpid: c.rpid, depth: u32::from(root > 0), has_later_sibling: false, reply_to: None };
        Ok((row, c))
    }

    /// Forgets the root comments kept for expanding threads (the videos were left).
    pub fn forget(&self) {
        self.roots.clear();
    }

    /// One page of root comments with their inline previews.
    pub async fn page(&self, area: CommentArea, by_time: bool, offset: Option<&str>) -> Result<CommentsPage> {
        let sort = if by_time { CommentSort::Time } else { CommentSort::Hot };
        let page = load_comments(self.api.as_ref(), &self.rules.rules(), area, sort, offset).await?;
        let threads = page
            .page
            .threads
            .iter()
            .map(|t| {
                self.roots.put(t.root.rpid, t.root.clone());
                thread_view(&t.root, &t.preview, usize::MAX, t.total_replies)
            })
            .collect();
        Ok(CommentsPage {
            pinned: page.page.pinned,
            threads,
            hidden_pinned: page.hidden_pinned,
            next_offset: page.page.next_offset,
        })
    }

    /// A thread of a loaded page in full, cut to its first `limit` replies.
    pub async fn thread(&self, area: CommentArea, root_rpid: u64, limit: usize) -> Result<ThreadView> {
        let root = self
            .roots
            .get(&root_rpid)
            .ok_or_else(|| CoreError::Invalid("unknown root comment; load the page first".into()))?;
        let replies = load_thread(self.api.as_ref(), area, root_rpid, THREAD_MAX_PAGES).await?;
        let total = root.reply_count.max(replies.len() as u32);
        Ok(thread_view(&root, &replies, limit, total))
    }

    /// The whole thread under `root_rpid`, even when it is on no loaded page (a notice's
    /// comment). Expanding it later works like any other thread.
    pub async fn focus(&self, area: CommentArea, root_rpid: u64) -> Result<ThreadView> {
        let root = self.api.root(area, root_rpid).await?;
        self.roots.put(root.rpid, root.clone());
        let replies = load_thread(self.api.as_ref(), area, root.rpid, THREAD_MAX_PAGES).await?;
        let total = root.reply_count.max(replies.len() as u32);
        Ok(thread_view(&root, &replies, usize::MAX, total))
    }
}

fn thread_view(root: &Comment, replies: &[Comment], limit: usize, total: u32) -> ThreadView {
    let (rows, hidden) = flatten_thread(root, replies, limit);
    let rows: Vec<(ThreadRow, Comment)> = rows
        .into_iter()
        .filter_map(|r| {
            let c = if r.rpid == root.rpid { Some(root) } else { replies.iter().find(|c| c.rpid == r.rpid) };
            c.map(|c| (r, c.clone()))
        })
        .collect();
    let shown = rows.len().saturating_sub(1) as u32;
    let hidden = (hidden as u32).max(total.saturating_sub(shown));
    ThreadView { root_rpid: root.rpid, rows, hidden, total_replies: total }
}

/// The picked people whose `@name` is still in `text`, each once.
fn mentioned(text: &str, picked: &[(String, u64)]) -> Vec<(String, u64)> {
    let mut out: Vec<(String, u64)> = Vec::new();
    for (name, mid) in picked {
        if !name.is_empty() && text.contains(&format!("@{name}")) && !out.iter().any(|(n, _)| n == name) {
            out.push((name.clone(), *mid));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sends_only_mentions_left_in_the_text() {
        let picked = vec![("小明".to_owned(), 1), ("阿杰".to_owned(), 2), ("小明".to_owned(), 1)];
        assert_eq!(mentioned("@小明 你看这个", &picked), vec![("小明".to_owned(), 1)]);
        assert!(mentioned("没有提到人", &picked).is_empty());
    }

    #[test]
    fn thread_counts_unloaded_replies_as_hidden() {
        let root = Comment { rpid: 1, reply_count: 10, ..Default::default() };
        let preview = vec![Comment { rpid: 2, root: 1, parent: 1, ..Default::default() }];
        let t = thread_view(&root, &preview, usize::MAX, 10);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.hidden, 9);
    }
}
