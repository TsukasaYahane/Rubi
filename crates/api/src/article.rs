//! 专栏 and 图文 bodies, both as typed paragraphs: a 图文 from its opus detail, a 专栏 (`cvN`)
//! from the article view, which carries the same paragraphs (old articles never moved to a
//! 图文 page, so their `read/cvN` link does not redirect anywhere).

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{Article, ArticleBlock, CommentPicture};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{image_of, str_of, u64_of};

const OPUS_DETAIL: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/opus/detail";
const ARTICLE_VIEW: &str = "https://api.bilibili.com/x/article/view";
/// The comment area `type` of 专栏 articles.
const ARTICLE_COMMENTS: u32 = 12;
/// What the web page asks for; the body comes back in its current form only with these.
const FEATURES: &str =
    "onlyfansVote,onlyfansAssetsV2,decorationCard,htmlNewStyle,ugcDelete,editable,opusPrivateVisible";

#[async_trait]
pub trait ArticleApi: Send + Sync {
    /// A 图文 post by its opus id.
    async fn opus(&self, id: &str) -> Result<Article>;
    /// A 专栏 article by its `cv` number.
    async fn article(&self, cvid: u64) -> Result<Article>;
}

pub struct WebArticle {
    http: Arc<HttpClient>,
}

impl WebArticle {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl ArticleApi for WebArticle {
    async fn opus(&self, id: &str) -> Result<Article> {
        let d = self
            .http
            .get_json(OPUS_DETAIL, params!["id" => id, "timezone_offset" => -480, "features" => FEATURES], true)
            .await?;
        Ok(article_of(&d["item"]))
    }

    async fn article(&self, cvid: u64) -> Result<Article> {
        let d = self.http.get_json(ARTICLE_VIEW, params!["id" => cvid], true).await?;
        let a = article_of_view(&d, cvid);
        if a.blocks.is_empty() && a.title.is_empty() {
            return Err(ApiError::Decode(format!("cv{cvid} has no readable body")));
        }
        Ok(a)
    }
}

/// `x/article/view`: title, author and banner of the article; the body from `opus.content`,
/// the same paragraphs a 图文 has.
pub(crate) fn article_of_view(d: &Value, cvid: u64) -> Article {
    let (blocks, raw): (Vec<ArticleBlock>, Vec<Value>) = d["opus"]["content"]["paragraphs"]
        .as_array()
        .map(|ps| ps.iter().filter_map(|p| block_of(p).map(|b| (b, p.clone()))).unzip())
        .unwrap_or_default();
    let id = Some(str_of(&d["dyn_id_str"])).filter(|s| !s.is_empty()).unwrap_or_else(|| format!("cv{cvid}"));
    Article {
        id,
        title: str_of(&d["title"]),
        author_name: str_of(&d["author"]["name"]),
        author_mid: u64_of(&d["author"]["mid"]),
        author_face: image_of(&d["author"]["face"]),
        published_at: u64_of(&d["publish_time"]) as i64,
        cover: image_of(&d["banner_url"]),
        blocks,
        comment_kind: ARTICLE_COMMENTS,
        comment_oid: cvid,
        hidden: 0,
        raw,
    }
}

fn module<'a>(item: &'a Value, kind: &str) -> &'a Value {
    item["modules"].as_array().and_then(|m| m.iter().find(|x| x["module_type"] == kind)).unwrap_or(&Value::Null)
}

pub(crate) fn article_of(item: &Value) -> Article {
    let author = &module(item, "MODULE_TYPE_AUTHOR")["module_author"];
    let (blocks, raw): (Vec<ArticleBlock>, Vec<Value>) =
        module(item, "MODULE_TYPE_CONTENT")["module_content"]["paragraphs"]
            .as_array()
            .map(|ps| ps.iter().filter_map(|p| block_of(p).map(|b| (b, p.clone()))).unzip())
            .unwrap_or_default();
    let cover = module(item, "MODULE_TYPE_TOP")["module_top"]["display"]["album"]["pics"]
        .as_array()
        .and_then(|p| p.first())
        .map(|p| image_of(&p["url"]))
        .unwrap_or_default();
    Article {
        id: str_of(&item["id_str"]),
        title: str_of(&module(item, "MODULE_TYPE_TITLE")["module_title"]["text"]),
        author_name: str_of(&author["name"]),
        author_mid: u64_of(&author["mid"]),
        author_face: image_of(&author["face"]),
        published_at: u64_of(&author["pub_ts"]) as i64,
        cover,
        blocks,
        comment_kind: u64_of(&item["basic"]["comment_type"]) as u32,
        comment_oid: u64_of(&item["basic"]["comment_id_str"]),
        hidden: 0,
        raw,
    }
}

/// `para_type`: 1 text, 2 pictures, 3 rule, 4 quote, 5 list, 6 link card, 8 heading, 9 heading
/// (as old articles write them). Anything else still shows its words, so new kinds degrade to
/// plain text instead of vanishing.
fn block_of(p: &Value) -> Option<ArticleBlock> {
    let block = match u64_of(&p["para_type"]) {
        2 => {
            let pics: Vec<CommentPicture> = p["pic"]["pics"]
                .as_array()?
                .iter()
                .map(|x| CommentPicture {
                    url: image_of(&x["url"]),
                    width: u64_of(&x["width"]) as u32,
                    height: u64_of(&x["height"]) as u32,
                })
                .filter(|x| !x.url.is_empty())
                .collect();
            return (!pics.is_empty()).then_some(ArticleBlock::Pictures(pics));
        }
        3 => return Some(ArticleBlock::Line),
        8 | 9 => ArticleBlock::Heading(words(p)),
        // A card's words are its title; promotion cards are left to the rule table.
        6 => ArticleBlock::Quote(first_title(&p["link_card"]).unwrap_or_else(|| words(p))),
        4 | 5 => ArticleBlock::Quote(words(p)),
        _ => ArticleBlock::Text(words(p)),
    };
    let empty = matches!(&block, ArticleBlock::Heading(t) | ArticleBlock::Text(t) | ArticleBlock::Quote(t) if t.trim().is_empty());
    (!empty).then_some(block)
}

/// The first non-empty `title` anywhere under `v`.
fn first_title(v: &Value) -> Option<String> {
    match v {
        Value::Object(map) => map
            .get("title")
            .and_then(Value::as_str)
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .or_else(|| map.values().find_map(first_title)),
        Value::Array(items) => items.iter().find_map(first_title),
        _ => None,
    }
}

/// Every word of a paragraph in order: `word.words` and `rich.text` of each text node, found
/// wherever the paragraph keeps its nodes (text, heading, quote, list items…).
fn words(p: &Value) -> String {
    let mut out = String::new();
    collect(p, &mut out);
    out
}

fn collect(v: &Value, out: &mut String) {
    match v {
        Value::Object(map) => {
            if let Some(nodes) = map.get("nodes").and_then(Value::as_array) {
                for n in nodes {
                    let w = str_of(&n["word"]["words"]);
                    out.push_str(if w.is_empty() { n["rich"]["text"].as_str().unwrap_or_default() } else { &w });
                }
                return;
            }
            for (k, child) in map {
                // Styling and pictures carry no words.
                if !matches!(k.as_str(), "format" | "pic" | "style") {
                    collect(child, out);
                }
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                if i > 0 && !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                collect(item, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn word(w: &str) -> Value {
        json!({"type": "TEXT_NODE_TYPE_WORD", "word": {"words": w, "style": {"bold": false}}})
    }

    /// Shapes from the 2026-11-01 capture of two opus pages (text, pictures, headings).
    #[test]
    fn reads_title_author_and_blocks() {
        let item = json!({"id_str": "1265393890559524872", "modules": [
            {"module_type": "MODULE_TYPE_TOP", "module_top": {"display": {"album": {"pics": [{"url": "http://i0.hdslb.com/c.png"}]}}}},
            {"module_type": "MODULE_TYPE_TITLE", "module_title": {"text": "标题"}},
            {"module_type": "MODULE_TYPE_AUTHOR", "module_author": {"name": "作者", "mid": 7, "pub_ts": "1793462353", "face": "https://i0.hdslb.com/f.jpg"}},
            {"module_type": "MODULE_TYPE_CONTENT", "module_content": {"paragraphs": [
                {"para_type": 8, "format": {"align": 0}, "heading": {"level": 3, "nodes": [word("小标题")]}},
                {"para_type": 1, "text": {"nodes": [word("第一句，"), {"type": "TEXT_NODE_TYPE_RICH", "rich": {"text": "链接"}}]}},
                {"para_type": 2, "pic": {"pics": [{"url": "http://i0.hdslb.com/p.png", "width": 1344, "height": 1792}]}},
                {"para_type": 1, "text": {"nodes": [word("  ")]}},
                {"para_type": 3, "line": {}},
                {"para_type": 5, "list": {"items": [{"nodes": [word("一")]}, {"nodes": [word("二")]}]}},
                {"para_type": 6, "link_card": {"card": {"type": "LINK_CARD_TYPE_GOODS", "goods": {"items": [{"title": "商品"}]}}}}
            ]}}
        ], "basic": {"comment_type": 12, "comment_id_str": "53699915"}});
        let a = article_of(&item);
        assert_eq!((a.title.as_str(), a.author_name.as_str(), a.published_at), ("标题", "作者", 1793462353));
        assert_eq!(a.cover, "https://i0.hdslb.com/c.png");
        assert_eq!(a.blocks[0], ArticleBlock::Heading("小标题".into()));
        assert_eq!(a.blocks[1], ArticleBlock::Text("第一句，链接".into()));
        assert!(
            matches!(&a.blocks[2], ArticleBlock::Pictures(p) if p[0].url == "https://i0.hdslb.com/p.png" && p[0].height == 1792)
        );
        assert_eq!(a.blocks[3], ArticleBlock::Line, "blank text is dropped");
        assert_eq!(a.blocks[4], ArticleBlock::Quote("一\n二".into()));
        assert_eq!(a.blocks[5], ArticleBlock::Quote("商品".into()));
        assert_eq!(a.raw.len(), a.blocks.len(), "each block keeps its paragraph");
        assert_eq!((a.comment_kind, a.comment_oid), (12, 53699915));
    }

    /// 2026-11-01: an old 专栏 (cv52157219) whose page never moved to 图文; type 9 is its heading.
    #[test]
    fn reads_an_article_view() {
        let d = json!({"title": "8月7日冒险者问专栏", "publish_time": 1786103813, "banner_url": "",
        "dyn_id_str": "1233795532663554056",
        "author": {"name": "星痕共鸣", "mid": 3546744874862955u64, "face": "https://i0.hdslb.com/f.jpg"},
        "opus": {"content": {"paragraphs": [
            {"para_type": 1, "text": {"nodes": [{"node_type": 1, "word": {"words": "冒险者，您好~"}}]}},
            {"para_type": 9, "format": {"heading_type": 2}, "text": {"nodes": [{"node_type": 1, "word": {"words": "一、关于溯光残因"}}]}}
        ]}}});
        let a = article_of_view(&d, 52157219);
        assert_eq!(
            (a.title.as_str(), a.author_name.as_str(), a.id.as_str()),
            ("8月7日冒险者问专栏", "星痕共鸣", "1233795532663554056")
        );
        assert_eq!(
            a.blocks,
            vec![ArticleBlock::Text("冒险者，您好~".into()), ArticleBlock::Heading("一、关于溯光残因".into())]
        );
        assert_eq!((a.comment_kind, a.comment_oid), (12, 52157219));
        assert_eq!(a.raw.len(), 2);
    }
}
