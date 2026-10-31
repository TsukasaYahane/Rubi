use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{Owner, SourceKind, Stats, VideoCard, Zone};
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{duration_of, image_of, plain_letters, str_of, u64_of};

const SEARCH_TYPE: &str = "https://api.bilibili.com/x/web-interface/wbi/search/type";
/// Hot words with their commercial flag (`stat_datas.is_commercial`).
const HOT: &str = "https://s.search.bilibili.com/main/hotword";
/// The same words without any flag; used only when `HOT` fails.
const HOT_SQUARE: &str = "https://api.bilibili.com/x/web-interface/search/square";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOrder {
    Relevance,
    Newest,
    MostPlayed,
    MostDanmaku,
    MostFavorited,
}

impl SearchOrder {
    fn param(self) -> &'static str {
        match self {
            SearchOrder::Relevance => "totalrank",
            SearchOrder::Newest => "pubdate",
            SearchOrder::MostPlayed => "click",
            SearchOrder::MostDanmaku => "dm",
            SearchOrder::MostFavorited => "stow",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HotWord {
    pub keyword: String,
    pub raw: Value,
}

#[async_trait]
pub trait SearchApi: Send + Sync {
    async fn videos(&self, keyword: &str, page: u32, order: SearchOrder) -> Result<Vec<VideoCard>>;
    /// Trending search words, unfiltered.
    async fn hot_words(&self, limit: u32) -> Result<Vec<HotWord>>;
}

pub struct WebSearch {
    http: Arc<HttpClient>,
}

impl WebSearch {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

/// Removes the `<em class="keyword">` highlight markup and common entities.
pub(crate) fn strip_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    decode_entities(&out)
}

/// Decodes the named entities the API uses plus numeric `&#NN;` / `&#xNN;` ones.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let decoded = tail.find(';').filter(|&end| end <= 10).and_then(|end| {
            let name = &tail[1..end];
            let ch = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                _ => name
                    .strip_prefix("#x")
                    .or_else(|| name.strip_prefix("#X"))
                    .map(|h| u32::from_str_radix(h, 16))
                    .or_else(|| name.strip_prefix('#').map(str::parse::<u32>))
                    .and_then(|r| r.ok())
                    .and_then(char::from_u32),
            };
            ch.map(|c| (c, end))
        });
        match decoded {
            Some((c, end)) => {
                out.push(c);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

pub(crate) fn card_of_search(v: &Value) -> VideoCard {
    let zone = match u64_of(&v["typeid"]) {
        0 => None,
        tid => Some(Zone { tid: tid as u32, name: str_of(&v["typename"]) }),
    };
    VideoCard {
        bvid: str_of(&v["bvid"]),
        aid: u64_of(&v["aid"]),
        cid: None,
        title: plain_letters(&strip_markup(&str_of(&v["title"]))),
        desc: str_of(&v["description"]),
        cover: image_of(&v["pic"]),
        owner: Owner { mid: u64_of(&v["mid"]), name: str_of(&v["author"]) },
        zone,
        tags: str_of(&v["tag"]).split(',').map(str::trim).filter(|t| !t.is_empty()).map(str::to_owned).collect(),
        duration_s: duration_of(&v["duration"]),
        pubdate: u64_of(&v["pubdate"]) as i64,
        stats: Stats {
            view: u64_of(&v["play"]),
            like: u64_of(&v["like"]),
            coin: 0,
            favorite: u64_of(&v["favorites"]),
            danmaku: u64_of(&v["danmaku"]),
            reply: u64_of(&v["review"]),
        },
        sources: vec![SourceKind::Subscription],
        raw: v.clone(),
    }
}

#[async_trait]
impl SearchApi for WebSearch {
    async fn videos(&self, keyword: &str, page: u32, order: SearchOrder) -> Result<Vec<VideoCard>> {
        let p = params!["search_type" => "video", "keyword" => keyword, "page" => page, "order" => order.param()];
        let d = self.http.get_json(SEARCH_TYPE, p, true).await?;
        Ok(d["result"].as_array().map(|a| a.iter().map(card_of_search).collect()).unwrap_or_default())
    }

    async fn hot_words(&self, limit: u32) -> Result<Vec<HotWord>> {
        let words = |list: &Value| -> Vec<HotWord> {
            list.as_array()
                .map(|a| a.iter().map(|w| HotWord { keyword: str_of(&w["keyword"]), raw: w.clone() }).collect())
                .unwrap_or_default()
        };
        match self.http.get_json(HOT, params!["limit" => limit], false).await {
            Ok(d) if d["list"].is_array() => Ok(words(&d["list"])),
            _ => {
                let d = self.http.get_json(HOT_SQUARE, params!["limit" => limit], false).await?;
                Ok(words(&d["trending"]["list"]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_search_result() {
        let v = json!({
            "type": "video", "aid": 1, "bvid": "BV1", "title": "<em class=\"keyword\">Rust</em> 入门 &amp; 实战",
            "author": "up", "mid": 2, "pic": "//i0.hdslb.com/x.jpg", "duration": "12:05", "play": 100,
            "typeid": "36", "typename": "科普", "tag": "rust,编程, ", "pubdate": 1700000000
        });
        let c = card_of_search(&v);
        assert_eq!(c.title, "Rust 入门 & 实战");
        assert_eq!(c.cover, "https://i0.hdslb.com/x.jpg");
        assert_eq!(c.duration_s, 725);
        assert_eq!(c.tags, vec!["rust", "编程"]);
        assert_eq!(c.zone.unwrap().tid, 36);
    }

    #[test]
    fn decodes_numeric_entities() {
        assert_eq!(strip_markup("Let&#x27;s &#39;go&#39; R&amp;D & more"), "Let's 'go' R&D & more");
    }
}
