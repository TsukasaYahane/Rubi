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

#[async_trait]
pub trait DynamicApi: Send + Sync {
    /// Followed UPs with recent posts, as the web dynamic page lists them. Login required.
    async fn followed_ups(&self) -> Result<Vec<FollowedUp>>;
    /// One UP's posts of every kind, newest first; pass the returned offset back for more.
    async fn up_dynamics(&self, mid: u64, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)>;
    /// Every followed UP's posts, newest first; pass the returned offset back for older ones.
    async fn followed_posts(&self, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)>;
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
        ..Default::default()
    };
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
        self.feed(params!["host_mid" => mid, "timezone_offset" => -480], offset).await
    }

    async fn followed_posts(&self, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        self.feed(params!["type" => "all", "timezone_offset" => -480], offset).await
    }
}

impl WebDynamic {
    /// A page of `feed/all` with `p`, after `offset`.
    async fn feed(&self, mut p: crate::http::Params, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        if let Some(o) = offset {
            p.push(("offset".into(), o.into()));
        }
        let d = self.http.get_json(FEED, p, false).await?;
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
