use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{Comment, CommentPicture, VideoLink, find_video_ref};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{image_of, plain_letters, str_of, u64_of};

const MAIN: &str = "https://api.bilibili.com/x/v2/reply/wbi/main";
const REPLIES: &str = "https://api.bilibili.com/x/v2/reply/reply";
const ACTION: &str = "https://api.bilibili.com/x/v2/reply/action";
const ADD: &str = "https://api.bilibili.com/x/v2/reply/add";

/// Which comment area: B站 keys every area by a `type` and an object id (`oid`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentArea {
    /// 1 video, 12 专栏, 17 图文…; whatever the content says it uses.
    pub kind: u32,
    pub oid: u64,
}

impl CommentArea {
    /// The comments under a video (by aid).
    pub fn video(aid: u64) -> Self {
        Self { kind: 1, oid: aid }
    }
}

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
    /// Root comments of an area.
    async fn page(&self, area: CommentArea, sort: CommentSort, offset: Option<&str>) -> Result<CommentPage>;
    /// One page of every reply under `root`, flat; rebuild the tree from `parent`.
    async fn replies(&self, area: CommentArea, root: u64, pn: u32, ps: u32) -> Result<(Vec<Comment>, u32)>;
    /// The root comment of a thread, for opening a thread that is not on a loaded page.
    async fn root(&self, area: CommentArea, root: u64) -> Result<Comment>;
    /// Likes a comment, or takes the like back. Login required.
    async fn like(&self, area: CommentArea, rpid: u64, on: bool) -> Result<()>;
    /// Replies to `parent` in the thread under `root` (both 0: a new root comment) and returns
    /// the reply as it will show. Login required.
    async fn reply(&self, area: CommentArea, root: u64, parent: u64, text: &str) -> Result<Comment>;
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
        emotes: v["content"]["emote"]
            .as_object()
            .map(|m| m.iter().map(|(k, e)| (k.clone(), image_of(&e["url"]))).filter(|(_, u)| !u.is_empty()).collect())
            .unwrap_or_default(),
        pictures: v["content"]["pictures"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|p| CommentPicture {
                        url: image_of(&p["img_src"]),
                        width: u64_of(&p["img_width"]) as u32,
                        height: u64_of(&p["img_height"]) as u32,
                    })
                    .filter(|p| !p.url.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        video_links: video_links(&v["content"]),
        note_cvid: u64_of(&v["note_cvid_str"]),
        liked: u64_of(&v["action"]) == 1,
        raw: v.clone(),
    }
}

/// A `jump_url` entry for a shop item: it carries a goods id, opens a shopping app, or is a
/// `b23.tv/mall-…` link. Short links look like any other `b23.tv` link, so this is what tells
/// them apart from videos.
fn is_goods(token: &str, j: &Value) -> bool {
    !j["extra"]["goods_item_id"].is_null()
        || !j["extra"]["goods_prefetched_cache"].is_null()
        || !str_of(&j["app_package_name"]).is_empty()
        || token.contains("/mall-")
        || token.contains("b23.tv/cm-")
}

/// `content.jump_url` entries that point at a video: the key is the text as written (a BV or
/// av id, or a video url); search highlights, shop items and other links are left out.
fn video_links(content: &Value) -> Vec<VideoLink> {
    let text = str_of(&content["message"]);
    content["jump_url"]
        .as_object()
        .map(|m| {
            m.iter()
                .filter(|(token, _)| text.contains(token.as_str()))
                .filter(|(token, j)| !is_goods(token, j))
                .filter_map(|(token, j)| {
                    let video = find_video_ref(token).or_else(|| find_video_ref(&str_of(&j["pc_url"])))?;
                    Some(VideoLink { token: token.clone(), title: plain_letters(&str_of(&j["title"])), video })
                })
                .collect()
        })
        .unwrap_or_default()
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
    async fn page(&self, area: CommentArea, sort: CommentSort, offset: Option<&str>) -> Result<CommentPage> {
        let mode = match sort {
            CommentSort::Hot => 3,
            CommentSort::Time => 2,
        };
        let pagination = serde_json::json!({ "offset": offset.unwrap_or_default() }).to_string();
        let p = params!["oid" => area.oid, "type" => area.kind, "mode" => mode, "pagination_str" => pagination, "plat" => 1, "web_location" => 1315875];
        Ok(page_of(&self.http.get_json(MAIN, p, true).await?))
    }

    async fn replies(&self, area: CommentArea, root: u64, pn: u32, ps: u32) -> Result<(Vec<Comment>, u32)> {
        let d = self
            .http
            .get_json(
                REPLIES,
                params!["oid" => area.oid, "type" => area.kind, "root" => root, "pn" => pn, "ps" => ps],
                false,
            )
            .await?;
        let up_mid = u64_of(&d["upper"]["mid"]);
        Ok((list(&d["replies"], up_mid), u64_of(&d["page"]["count"]) as u32))
    }

    async fn root(&self, area: CommentArea, root: u64) -> Result<Comment> {
        let d = self
            .http
            .get_json(
                REPLIES,
                params!["oid" => area.oid, "type" => area.kind, "root" => root, "pn" => 1, "ps" => 1],
                false,
            )
            .await?;
        if d["root"].is_null() {
            return Err(ApiError::Decode("reply/reply has no root comment".into()));
        }
        Ok(comment_of(&d["root"], u64_of(&d["upper"]["mid"])))
    }

    async fn like(&self, area: CommentArea, rpid: u64, on: bool) -> Result<()> {
        let form = params!["type" => area.kind, "oid" => area.oid, "rpid" => rpid, "action" => u8::from(on)];
        self.http.post_form(ACTION, form).await.map(|_| ())
    }

    async fn reply(&self, area: CommentArea, root: u64, parent: u64, text: &str) -> Result<Comment> {
        let mut form = params!["type" => area.kind, "oid" => area.oid, "message" => text, "plat" => 1];
        if root > 0 {
            form.push(("root".into(), root.to_string()));
            form.push(("parent".into(), parent.max(root).to_string()));
        }
        let d = self.http.post_form(ADD, form).await?;
        Ok(comment_of(&d["reply"], 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 2026-11-01: an UP's pinned shop comment (BV154nE6FEK1) links `b23.tv/mall-…` items that
    /// open Taobao; they are not videos.
    #[test]
    fn shop_links_are_not_video_links() {
        let v = json!({"rpid": 1, "content": {
            "message": "推荐这把椅子 https://b23.tv/mall-1PEUvH-75mBT 看 BV1zpxy6xEFC",
            "jump_url": {
                "https://b23.tv/mall-1PEUvH-75mBT": {"title": "工学椅", "app_package_name": "com.taobao.taobao", "extra": {"goods_item_id": 104811215}},
                "BV1zpxy6xEFC": {"title": "视频", "pc_url": ""}}}});
        let c = comment_of(&v, 0);
        assert_eq!(c.video_links.len(), 1);
        assert_eq!(c.video_links[0].token, "BV1zpxy6xEFC");
    }

    /// Shapes from the 2026-10-31 capture of reply/main.
    #[test]
    fn parses_pictures_notes_and_video_links() {
        let v = json!({"rpid": 1, "mid": 2, "note_cvid_str": "21936142", "content": {
            "message": "看看 BV1zpxy6xEFC 再看 #话题",
            "pictures": [{"img_src": "http://i0.hdslb.com/bfs/new_dyn/a.gif", "img_width": 240, "img_height": 240}],
            "jump_url": {"BV1zpxy6xEFC": {"title": "游戏王决斗链接真好玩", "pc_url": ""},
                         "话题": {"title": "话题", "pc_url": "//search.bilibili.com/all?keyword=x", "extra": {"is_word_search": true}}}}});
        let c = comment_of(&v, 0);
        assert_eq!(c.pictures[0].url, "https://i0.hdslb.com/bfs/new_dyn/a.gif");
        assert_eq!(c.note_cvid, 21936142);
        assert_eq!(c.video_links.len(), 1);
        assert_eq!(c.video_links[0].token, "BV1zpxy6xEFC");
        assert_eq!(c.video_links[0].title, "游戏王决斗链接真好玩");
    }

    #[test]
    fn parses_page_with_pinned_and_previews() {
        let d = json!({
            "upper": {"mid": 9, "top": {"rpid": 1, "mid": 9, "member": {"uname": "up"}, "content": {"message": "置顶", "jump_url": {"x": {}}}}},
            "top_replies": [],
            "replies": [{
                "rpid": 2, "root": 0, "parent": 0, "mid": 5, "like": 3, "rcount": 4, "ctime": 100,
                "member": {"uname": "a"}, "content": {"message": "hi[doge]", "emote": {"[doge]": {"url": "https://e/doge.png"}}},
                "replies": [{"rpid": 3, "root": 2, "parent": 2, "mid": 9, "member": {"uname": "up"}, "content": {"message": "yo"}}]
            }],
            "cursor": {"is_end": false, "pagination_reply": {"next_offset": "abc"}}
        });
        let p = page_of(&d);
        assert_eq!(p.pinned.len(), 1);
        assert!(p.pinned[0].pinned && p.pinned[0].is_up);
        assert_eq!(p.threads[0].total_replies, 4);
        assert_eq!(p.threads[0].root.emotes["[doge]"], "https://e/doge.png");
        assert!(p.threads[0].preview[0].is_up);
        assert_eq!(p.threads[0].preview[0].parent, 2);
        assert_eq!(p.next_offset.as_deref(), Some("abc"));
    }
}
