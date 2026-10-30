use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::Comment;
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{str_of, u64_of};

const MAIN: &str = "https://api.bilibili.com/x/v2/reply/wbi/main";
const REPLIES: &str = "https://api.bilibili.com/x/v2/reply/reply";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentSort {
    Hot,
    Time,
}

/// A root comment with the few replies the list endpoint inlines.
#[derive(Debug, Clone, Default)]
pub struct ThreadPreview {
    pub root: Comment,
    pub preview: Vec<Comment>,
    pub total_replies: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CommentPage {
    /// Pinned comments, unfiltered.
    pub pinned: Vec<Comment>,
    pub threads: Vec<ThreadPreview>,
    pub up_mid: u64,
    /// Pass back to get the next page; `None` at the end.
    pub next_offset: Option<String>,
}

#[async_trait]
pub trait CommentApi: Send + Sync {
    /// Root comments of a video (`oid` is the aid).
    async fn page(&self, aid: u64, sort: CommentSort, offset: Option<&str>) -> Result<CommentPage>;
    /// One page of every reply under `root`, flat; rebuild the tree from `parent`.
    async fn replies(&self, aid: u64, root: u64, pn: u32, ps: u32) -> Result<(Vec<Comment>, u32)>;
}

pub struct WebComment {
    http: Arc<HttpClient>,
}

impl WebComment {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

pub(crate) fn comment_of(v: &Value, up_mid: u64) -> Comment {
    let mid = u64_of(&v["mid"]);
    Comment {
        rpid: u64_of(&v["rpid"]),
        root: u64_of(&v["root"]),
        parent: u64_of(&v["parent"]),
        mid,
        name: str_of(&v["member"]["uname"]),
        text: str_of(&v["content"]["message"]),
        likes: u64_of(&v["like"]),
        ctime: u64_of(&v["ctime"]) as i64,
        is_up: up_mid != 0 && mid == up_mid,
        reply_count: u64_of(&v["rcount"]) as u32,
        pinned: false,
        raw: v.clone(),
    }
}

fn list(v: &Value, up_mid: u64) -> Vec<Comment> {
    v.as_array().map(|a| a.iter().map(|c| comment_of(c, up_mid)).collect()).unwrap_or_default()
}

pub(crate) fn page_of(d: &Value) -> CommentPage {
    let up_mid = u64_of(&d["upper"]["mid"]);
    let mut pinned: Vec<Comment> = list(&d["top_replies"], up_mid);
    if d["upper"]["top"].is_object() {
        let top = comment_of(&d["upper"]["top"], up_mid);
        if !pinned.iter().any(|p| p.rpid == top.rpid) {
            pinned.push(top);
        }
    }
    for p in &mut pinned {
        p.pinned = true;
    }
    let threads = d["replies"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|r| ThreadPreview {
                    root: comment_of(r, up_mid),
                    preview: list(&r["replies"], up_mid),
                    total_replies: u64_of(&r["rcount"]) as u32,
                })
                .collect()
        })
        .unwrap_or_default();
    let cursor = &d["cursor"];
    let next = str_of(&cursor["pagination_reply"]["next_offset"]);
    let next_offset = (!cursor["is_end"].as_bool().unwrap_or(true) && !next.is_empty()).then_some(next);
    CommentPage { pinned, threads, up_mid, next_offset }
}

#[async_trait]
impl CommentApi for WebComment {
    async fn page(&self, aid: u64, sort: CommentSort, offset: Option<&str>) -> Result<CommentPage> {
        let mode = match sort {
            CommentSort::Hot => 3,
            CommentSort::Time => 2,
        };
        let pagination = serde_json::json!({ "offset": offset.unwrap_or_default() }).to_string();
        let p = params!["oid" => aid, "type" => 1, "mode" => mode, "pagination_str" => pagination, "plat" => 1, "web_location" => 1315875];
        Ok(page_of(&self.http.get_json(MAIN, p, true).await?))
    }

    async fn replies(&self, aid: u64, root: u64, pn: u32, ps: u32) -> Result<(Vec<Comment>, u32)> {
        let d = self
            .http
            .get_json(REPLIES, params!["oid" => aid, "type" => 1, "root" => root, "pn" => pn, "ps" => ps], false)
            .await?;
        let up_mid = u64_of(&d["upper"]["mid"]);
        Ok((list(&d["replies"], up_mid), u64_of(&d["page"]["count"]) as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_page_with_pinned_and_previews() {
        let d = json!({
            "upper": {"mid": 9, "top": {"rpid": 1, "mid": 9, "member": {"uname": "up"}, "content": {"message": "置顶", "jump_url": {"x": {}}}}},
            "top_replies": [],
            "replies": [{
                "rpid": 2, "root": 0, "parent": 0, "mid": 5, "like": 3, "rcount": 4, "ctime": 100,
                "member": {"uname": "a"}, "content": {"message": "hi"},
                "replies": [{"rpid": 3, "root": 2, "parent": 2, "mid": 9, "member": {"uname": "up"}, "content": {"message": "yo"}}]
            }],
            "cursor": {"is_end": false, "pagination_reply": {"next_offset": "abc"}}
        });
        let p = page_of(&d);
        assert_eq!(p.pinned.len(), 1);
        assert!(p.pinned[0].pinned && p.pinned[0].is_up);
        assert_eq!(p.threads[0].total_replies, 4);
        assert!(p.threads[0].preview[0].is_up);
        assert_eq!(p.threads[0].preview[0].parent, 2);
        assert_eq!(p.next_offset.as_deref(), Some("abc"));
    }
}
