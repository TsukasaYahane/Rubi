//! Comments under a video.

use rubi_model::ThreadRow;

use crate::{FfiCommentPage, FfiCommentRef, FfiCommentRow, FfiCommentThread, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// Root comments with their inline previews drawn as fishbone rows.
    pub async fn comments(&self, aid: u64, by_time: bool, offset: Option<String>) -> Res<FfiCommentPage> {
        let page = self.comments.page(aid, by_time, offset.as_deref()).await?;
        let pinned = page
            .pinned
            .iter()
            .map(|c| {
                let row = ThreadRow { rpid: c.rpid, depth: 0, has_later_sibling: false, reply_to: None };
                FfiCommentRow::of(&row, c)
            })
            .collect();
        Ok(FfiCommentPage {
            pinned,
            threads: page.threads.iter().map(Into::into).collect(),
            hidden_pinned: page.hidden_pinned as u32,
            next_offset: page.next_offset,
        })
    }

    /// Loads a thread fully and returns its first `limit` replies as fishbone rows.
    pub async fn comment_thread(&self, aid: u64, root_rpid: u64, limit: u32) -> Res<FfiCommentThread> {
        Ok((&self.comments.thread(aid, root_rpid, limit as usize).await?).into())
    }

    /// The whole thread holding a comment from a notice, root first, even when it is not on
    /// any loaded page. "再看 5 条" then works on it like on any other thread.
    pub async fn focus_thread(&self, aid: u64, comment: FfiCommentRef) -> Res<FfiCommentThread> {
        Ok((&self.comments.focus(aid, comment.root).await?).into())
    }
}
