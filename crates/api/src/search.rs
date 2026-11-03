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

/// An UP found by name. B站 matches aliases too: 「TES」 finds 滔搏电子竞技俱乐部.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserHit {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub fans: u64,
    pub videos: u64,
    pub level: u32,
    /// The live room when on air, else 0.
    pub live_room: u64,
}

/// One page of UPs and whether more follow.
#[derive(Debug, Clone, Default)]
pub struct UserHits {
    pub users: Vec<UserHit>,
    pub has_more: bool,
}

#[async_trait]
pub trait SearchApi: Send + Sync {
    async fn videos(&self, keyword: &str, page: u32, order: SearchOrder) -> Result<Vec<VideoCard>>;
    /// Page `page` (from 1) of UPs whose name (or alias) fits.
    async fn users(&self, keyword: &str, page: u32) -> Result<UserHits>;
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

/// `search/type?search_type=bili_user`.
pub(crate) fn users_of(d: &Value, page: u32) -> UserHits {
    let users: Vec<UserHit> = d["result"]
        .as_array()
        .map(|l| {
            l.iter()
                .map(|u| UserHit {
                    mid: u64_of(&u["mid"]),
                    name: strip_markup(u["uname"].as_str().unwrap_or_default()),
                    face: image_of(&u["upic"]),
                    sign: str_of(&u["usign"]),
                    fans: u64_of(&u["fans"]),
                    videos: u64_of(&u["videos"]),
                    level: u64_of(&u["level"]) as u32,
                    live_room: if u64_of(&u["is_live"]) == 1 { u64_of(&u["room_id"]) } else { 0 },
                })
                .filter(|u| u.mid > 0)
                .collect()
        })
        .unwrap_or_default();
    let has_more = !users.is_empty() && u64_of(&d["numPages"]) > u64::from(page);
    UserHits { users, has_more }
}

#[async_trait]
impl SearchApi for WebSearch {
    async fn videos(&self, keyword: &str, page: u32, order: SearchOrder) -> Result<Vec<VideoCard>> {
        let p = params!["search_type" => "video", "keyword" => keyword, "page" => page, "order" => order.param()];
        let d = self.http.get_json(SEARCH_TYPE, p, true).await?;
        Ok(d["result"].as_array().map(|a| a.iter().map(card_of_search).collect()).unwrap_or_default())
    }

    async fn users(&self, keyword: &str, page: u32) -> Result<UserHits> {
        let p = params!["search_type" => "bili_user", "keyword" => keyword, "page" => page.max(1)];
        let d = self.http.get_json(SEARCH_TYPE, p, true).await?;
        Ok(users_of(&d, page.max(1)))
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

    /// 2026-11-02: 「TES」 finds the club by its alias first.
    #[test]
    fn reads_found_users() {
        let d = serde_json::json!({"numPages": 3, "result": [
            {"mid": 300634025, "uname": "滔搏电子竞技俱乐部", "upic": "//i0.hdslb.com/f.jpg", "usign": "欢迎关注", "fans": 955303,
             "videos": 1200, "level": 6, "is_live": 1, "room_id": 22},
            {"mid": 12524890, "uname": "<em class=\"keyword\">TES</em>首席观察员", "fans": 18352, "is_live": 0, "room_id": 5}]});
        let p = users_of(&d, 1);
        assert!(p.has_more);
        assert_eq!(
            (p.users[0].name.as_str(), p.users[0].live_room, p.users[0].face.as_str()),
            ("滔搏电子竞技俱乐部", 22, "https://i0.hdslb.com/f.jpg")
        );
        assert_eq!((p.users[1].name.as_str(), p.users[1].live_room), ("TES首席观察员", 0));
        assert!(!users_of(&d, 3).has_more);
    }

    #[test]
    fn decodes_numeric_entities() {
        assert_eq!(strip_markup("Let&#x27;s &#39;go&#39; R&amp;D & more"), "Let's 'go' R&D & more");
    }
}
