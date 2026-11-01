//! Comments under a video.

use rubi_api::CommentArea;
use rubi_model::ThreadRow;

use crate::{FfiCommentArea, FfiCommentPage, FfiCommentRef, FfiCommentRow, FfiCommentThread, Res, RubiCore};

fn area_of(a: FfiCommentArea) -> CommentArea {
    CommentArea { kind: a.kind, oid: a.oid }
}

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// The comment area under a video.
    pub fn video_comments(&self, aid: u64) -> FfiCommentArea {
        FfiCommentArea { kind: CommentArea::video(aid).kind, oid: aid }
    }

    /// Root comments with their inline previews drawn as fishbone rows.
    pub async fn comments(&self, area: FfiCommentArea, by_time: bool, offset: Option<String>) -> Res<FfiCommentPage> {
        let page = self.comments.page(area_of(area), by_time, offset.as_deref()).await?;
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
    pub async fn comment_thread(&self, area: FfiCommentArea, root_rpid: u64, limit: u32) -> Res<FfiCommentThread> {
        Ok((&self.comments.thread(area_of(area), root_rpid, limit as usize).await?).into())
    }

    /// Likes a comment, or takes the like back. Login required.
    pub async fn like_comment(&self, area: FfiCommentArea, rpid: u64, on: bool) -> Res<()> {
        Ok(self.comments.like(area_of(area), rpid, on).await?)
    }

    /// Replies to `parent` in the thread under `root` (both 0: a new comment under the video);
    /// returns the reply as a row to show at once. Login required.
    pub async fn reply_comment(
        &self,
        area: FfiCommentArea,
        root: u64,
        parent: u64,
        text: String,
    ) -> Res<FfiCommentRow> {
        let (row, c) = self.comments.reply(area_of(area), root, parent, &text).await?;
        Ok(FfiCommentRow::of(&row, &c))
    }

    /// The whole thread holding a comment from a notice, root first, even when it is not on
    /// any loaded page. "再看 5 条" then works on it like on any other thread.
    pub async fn focus_thread(&self, area: FfiCommentArea, comment: FfiCommentRef) -> Res<FfiCommentThread> {
        Ok((&self.comments.focus(area_of(area), comment.root).await?).into())
    }
}
