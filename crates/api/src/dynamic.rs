//! 动态: the followed UPs bar and one UP's posts of every kind.

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{Dynamic, DynamicKind, FollowedUp};
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{card_of_dynamic, https_image, image_of, str_of, u64_of};

const PORTAL: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/portal";
const FEED: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/all";
/// `feed/all` with `host_mid` only knows followed UPs; this one knows everyone.
const SPACE: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/space";
const SPACE_SEARCH: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/space/search";

#[async_trait]
pub trait DynamicApi: Send + Sync {
    /// Followed UPs with recent posts, as the web dynamic page lists them. Login required.
    async fn followed_ups(&self) -> Result<Vec<FollowedUp>>;
    /// One UP's posts of every kind, newest first; pass the returned offset back for more.
    async fn up_dynamics(&self, mid: u64, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)>;
    /// Every followed UP's posts, newest first; pass the returned offset back for older ones.
    async fn followed_posts(&self, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)>;
    /// Page `page` (from 1) of an UP's posts matching `keyword` (videos, 专栏 and posts alike),
    /// and whether more follow.
    async fn search_up(&self, mid: u64, keyword: &str, page: u32) -> Result<(Vec<Dynamic>, bool)>;
}

pub struct WebDynamic {
    http: Arc<HttpClient>,
}

impl WebDynamic {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

/// `//host/path` and `http://` links become `https://`.
fn link_of(v: &Value) -> String {
    https_image(&str_of(v))
}

/// The emote images among a text's rich nodes, by token.
fn emotes_of(nodes: &Value, into: &mut std::collections::BTreeMap<String, String>) {
    for node in nodes.as_array().into_iter().flatten() {
        if str_of(&node["type"]) != "RICH_TEXT_NODE_TYPE_EMOJI" {
            continue;
        }
        let token = Some(str_of(&node["orig_text"]))
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| str_of(&node["emoji"]["text"]));
        let url = image_of(&node["emoji"]["icon_url"]);
        if token.starts_with('[') && !url.is_empty() {
            into.insert(token, url);
        }
    }
}

/// The words of a rich-text node list, joined as they read.
pub(crate) fn nodes_text(nodes: &Value) -> String {
    nodes
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|n| {
                    let orig = str_of(&n["orig_text"]);
                    if orig.is_empty() { str_of(&n["text"]) } else { orig }
                })
                .collect::<String>()
        })
        .unwrap_or_default()
}

pub(crate) fn dynamic_of(item: &Value) -> Dynamic {
    let author = &item["modules"]["module_author"];
    let md = &item["modules"]["module_dynamic"];
    let major = &md["major"];
    let mut d = Dynamic {
        id: str_of(&item["id_str"]),
        author_mid: u64_of(&author["mid"]),
        author_name: str_of(&author["name"]),
        author_face: image_of(&author["face"]),
        published_at: u64_of(&author["pub_ts"]) as i64,
        text: str_of(&md["desc"]["text"]),
        comment_kind: u64_of(&item["basic"]["comment_type"]) as u32,
        comment_oid: u64_of(&item["basic"]["comment_id_str"]),
        comment_count: u64_of(&item["modules"]["module_stat"]["comment"]["count"]),
        ..Default::default()
    };
    emotes_of(&md["desc"]["rich_text_nodes"], &mut d.emotes);
    emotes_of(&major["opus"]["summary"]["rich_text_nodes"], &mut d.emotes);
    d.kind = match str_of(&item["type"]).as_str() {
        "DYNAMIC_TYPE_AV" => DynamicKind::Video,
        "DYNAMIC_TYPE_WORD" => DynamicKind::Word,
        "DYNAMIC_TYPE_DRAW" => DynamicKind::Draw,
        "DYNAMIC_TYPE_ARTICLE" => DynamicKind::Article,
        "DYNAMIC_TYPE_FORWARD" => DynamicKind::Forward,
        _ => DynamicKind::Other,
    };
    match str_of(&major["type"]).as_str() {
        "MAJOR_TYPE_ARCHIVE" => {
            d.video = card_of_dynamic(item);
            d.kind = DynamicKind::Video;
        }
        "MAJOR_TYPE_DRAW" => {
            d.pictures = pics(&major["draw"]["items"], "src");
        }
        "MAJOR_TYPE_OPUS" => {
            let o = &major["opus"];
            d.kind = if d.kind == DynamicKind::Article { DynamicKind::Article } else { DynamicKind::Opus };
            d.title = str_of(&o["title"]);
            if d.text.is_empty() {
                d.text = str_of(&o["summary"]["text"]);
            }
            d.pictures = pics(&o["pics"], "url");
            d.link = link_of(&o["jump_url"]);
        }
        "MAJOR_TYPE_ARTICLE" => {
            let a = &major["article"];
            d.kind = DynamicKind::Article;
            d.title = str_of(&a["title"]);
            if d.text.is_empty() {
                d.text = str_of(&a["desc"]);
            }
            d.pictures = a["covers"].as_array().map(|c| c.iter().map(image_of).collect()).unwrap_or_default();
            d.link = link_of(&a["jump_url"]);
        }
        "MAJOR_TYPE_COMMON" => {
            let c = &major["common"];
            d.title = str_of(&c["title"]);
            if d.text.is_empty() {
                d.text = str_of(&c["desc"]);
            }
            d.pictures = Some(image_of(&c["cover"])).filter(|u| !u.is_empty()).into_iter().collect();
            d.link = link_of(&c["jump_url"]);
        }
        _ => {}
    }
    // Whichever shape came back, the words may sit somewhere other than desc.text: the 图文
    // summary, a 专栏's description, a forwarded post's own text.
    if d.text.is_empty() {
        let elsewhere = [
            &md["major"]["opus"]["summary"]["text"],
            &md["major"]["article"]["desc"],
            &md["major"]["common"]["desc"],
        ];
        for value in elsewhere {
            let words = str_of(value);
            if !words.is_empty() {
                d.text = words;
                break;
            }
        }
    }
    // The text field is sometimes empty while the nodes that build it are not.
    if d.text.is_empty() {
        d.text = nodes_text(&md["desc"]["rich_text_nodes"]);
    }
    if d.text.is_empty() {
        d.text = nodes_text(&md["major"]["opus"]["summary"]["rich_text_nodes"]);
    }
    if d.text.is_empty()
        && let Some(orig) = item["orig"].as_object()
    {
        let forwarded = str_of(&orig["modules"]["module_dynamic"]["desc"]["text"]);
        if !forwarded.is_empty() {
            d.text = forwarded;
        }
    }
    if !item["orig"].is_null() {
        d.orig = Some(Box::new(dynamic_of(&item["orig"])));
    }
    d
}

fn pics(list: &Value, key: &str) -> Vec<String> {
    list.as_array().map(|a| a.iter().map(|p| image_of(&p[key])).filter(|u| !u.is_empty()).collect()).unwrap_or_default()
}

#[async_trait]
impl DynamicApi for WebDynamic {
    async fn followed_ups(&self) -> Result<Vec<FollowedUp>> {
        let d = self.http.get_json(PORTAL, params![], false).await?;
        Ok(d["up_list"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|u| FollowedUp {
                        mid: u64_of(&u["mid"]),
                        name: str_of(&u["uname"]),
                        face: image_of(&u["face"]),
                        has_update: u["has_update"].as_bool().unwrap_or(false),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn up_dynamics(&self, mid: u64, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        self.feed(SPACE, params!["host_mid" => mid, "timezone_offset" => -480], offset, true).await
    }

    async fn followed_posts(&self, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        self.feed(FEED, params!["type" => "all", "timezone_offset" => -480], offset, false).await
    }

    async fn search_up(&self, mid: u64, keyword: &str, page: u32) -> Result<(Vec<Dynamic>, bool)> {
        let d = self
            .http
            .get_json(SPACE_SEARCH, params!["host_mid" => mid, "keyword" => keyword, "page" => page.max(1)], false)
            .await?;
        let items = d["items"].as_array().map(|a| a.iter().map(dynamic_of).collect()).unwrap_or_default();
        Ok((items, d["has_more"].as_bool().unwrap_or(false)))
    }
}

impl WebDynamic {
    /// A page of the feed at `url` with `p`, after `offset`.
    async fn feed(
        &self,
        url: &str,
        mut p: crate::http::Params,
        offset: Option<&str>,
        sign: bool,
    ) -> Result<(Vec<Dynamic>, Option<String>)> {
        if let Some(o) = offset {
            p.push(("offset".into(), o.into()));
        }
        let d = self.http.get_json(url, p, sign).await?;
        let items = d["items"].as_array().map(|a| a.iter().map(dynamic_of).collect()).unwrap_or_default();
        let next = Some(str_of(&d["offset"])).filter(|o| !o.is_empty() && d["has_more"].as_bool().unwrap_or(false));
        Ok((items, next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Shapes from the 2026-11-01 capture of feed/all?host_mid=…
    #[test]
    fn emotes_come_with_the_text() {
        let item = json!({"id_str": "9", "type": "DYNAMIC_TYPE_WORD", "modules": {"module_author": {"mid": 1, "name": "u"},
            "module_dynamic": {"desc": {"text": "好[doge]", "rich_text_nodes": [
                {"type": "RICH_TEXT_NODE_TYPE_TEXT", "orig_text": "好", "text": "好"},
                {"type": "RICH_TEXT_NODE_TYPE_EMOJI", "orig_text": "[doge]", "text": "[doge]",
                 "emoji": {"icon_url": "http://i0.hdslb.com/bfs/emote/doge.png", "text": "[doge]"}}]}, "major": null}}});
        let d = dynamic_of(&item);
        assert_eq!(d.emotes.get("[doge]").map(String::as_str), Some("https://i0.hdslb.com/bfs/emote/doge.png"));
    }

    #[test]
    fn parses_kinds() {
        let author = json!({"mid": 7, "name": "up", "face": "http://i0.hdslb.com/f.jpg", "pub_ts": "1793251346"});
        let draw = json!({"id_str": "1", "type": "DYNAMIC_TYPE_DRAW", "modules": {"module_author": author,
            "module_dynamic": {"desc": {"text": "看图"}, "major": {"type": "MAJOR_TYPE_DRAW", "draw": {"items": [{"src": "http://i0.hdslb.com/a.jpg"}]}}}}});
        let d = dynamic_of(&draw);
        assert_eq!(
            (d.kind, d.text.as_str(), d.pictures[0].as_str()),
            (DynamicKind::Draw, "看图", "https://i0.hdslb.com/a.jpg")
        );
        assert_eq!((d.author_mid, d.published_at), (7, 1793251346));

        let forward = json!({"id_str": "2", "type": "DYNAMIC_TYPE_FORWARD", "modules": {"module_author": author,
            "module_dynamic": {"desc": {"text": "转发说两句"}, "major": null}}, "orig": draw});
        let f = dynamic_of(&forward);
        assert_eq!(f.kind, DynamicKind::Forward);
        let commented = json!({"id_str": "5", "basic": {"comment_type": 17, "comment_id_str": "1265"},
            "modules": {"module_author": author, "module_dynamic": {}, "module_stat": {"comment": {"count": 12}}}});
        let c = dynamic_of(&commented);
        assert_eq!((c.comment_kind, c.comment_oid, c.comment_count), (17, 1265, 12));
        assert_eq!(f.orig.as_ref().unwrap().kind, DynamicKind::Draw);

        let video = json!({"id_str": "3", "type": "DYNAMIC_TYPE_AV", "modules": {"module_author": author, "module_dynamic": {"desc": null,
            "major": {"type": "MAJOR_TYPE_ARCHIVE", "archive": {"bvid": "BV1tHsZ66EXU", "aid": "1", "title": "t", "cover": "http://c.jpg",
                "duration_text": "01:12", "stat": {"play": "1.2万", "danmaku": "8"}}}}}});
        let v = dynamic_of(&video);
        assert_eq!(v.video.as_ref().unwrap().stats.view, 12_000);

        let opus = json!({"id_str": "4", "type": "DYNAMIC_TYPE_DRAW", "modules": {"module_author": author, "module_dynamic": {"desc": null,
            "major": {"type": "MAJOR_TYPE_OPUS", "opus": {"title": "标题", "summary": {"text": "摘要"}, "pics": [{"url": "//i0.hdslb.com/p.jpg"}],
                "jump_url": "//www.bilibili.com/opus/4"}}}}});
        let o = dynamic_of(&opus);
        assert_eq!((o.kind, o.title.as_str(), o.text.as_str()), (DynamicKind::Opus, "标题", "摘要"));
        assert_eq!(o.link, "https://www.bilibili.com/opus/4");
    }
}
