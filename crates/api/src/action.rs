//! Account actions (like / coin / favourite / watch later / follow) and the user's
//! library (history, watch-later list, favourite folders, followings). Login required.

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{FavFolder, Following, HistoryEntry, Owner, SourceKind, Stats, VideoCard, VideoRelation};
use serde_json::Value;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{card_of, duration_of, image_of, str_of, title_of, u64_of};

const LIKE: &str = "https://api.bilibili.com/x/web-interface/archive/like";
const TRIPLE: &str = "https://api.bilibili.com/x/web-interface/archive/like/triple";
const COIN: &str = "https://api.bilibili.com/x/web-interface/coin/add";
const FAV_DEAL: &str = "https://api.bilibili.com/x/v3/fav/resource/deal";
const FAV_FOLDERS: &str = "https://api.bilibili.com/x/v3/fav/folder/created/list-all";
const FAV_LIST: &str = "https://api.bilibili.com/x/v3/fav/resource/list";
const FOLDER_ADD: &str = "https://api.bilibili.com/x/v3/fav/folder/add";
const FOLDER_EDIT: &str = "https://api.bilibili.com/x/v3/fav/folder/edit";
const FOLDER_DEL: &str = "https://api.bilibili.com/x/v3/fav/folder/del";
const FOLDER_INFO: &str = "https://api.bilibili.com/x/v3/fav/folder/info";
const FAV_BATCH_DEL: &str = "https://api.bilibili.com/x/v3/fav/resource/batch-del";
const FAV_MOVE: &str = "https://api.bilibili.com/x/v3/fav/resource/move";
const FAV_COPY: &str = "https://api.bilibili.com/x/v3/fav/resource/copy";
const TOVIEW_ADD: &str = "https://api.bilibili.com/x/v2/history/toview/add";
const TOVIEW_LIST: &str = "https://api.bilibili.com/x/v2/history/toview";
const TOVIEW_DEL: &str = "https://api.bilibili.com/x/v2/history/toview/del";
const RELATION_MODIFY: &str = "https://api.bilibili.com/x/relation/modify";
const ARCHIVE_RELATION: &str = "https://api.bilibili.com/x/web-interface/archive/relation";
const HISTORY: &str = "https://api.bilibili.com/x/web-interface/history/cursor";
const FOLLOWINGS: &str = "https://api.bilibili.com/x/relation/followings";
const HEARTBEAT: &str = "https://api.bilibili.com/x/click-interface/web/heartbeat";

/// What the video page does to a video and its uploader: like, coin, 三连, favourite,
/// 稍后再看, follow. Used by `Actions` only.
#[async_trait]
pub trait VideoActionApi: Send + Sync {
    async fn relation(&self, bvid: &str) -> Result<VideoRelation>;
    async fn like(&self, bvid: &str, on: bool) -> Result<()>;
    /// `count` is 1 or 2.
    async fn coin(&self, bvid: &str, count: u32, also_like: bool) -> Result<()>;
    /// 一键三连: like, coins and the default favourite folder at once. A video already coined
    /// gets no more coins (verified 2026-10-31: `coin` succeeds with `multiply` 0).
    async fn triple(&self, bvid: &str) -> Result<Triple>;
    /// The user's own folders, each marked if it holds the video `aid`.
    async fn fav_folders(&self, aid: u64) -> Result<Vec<FavFolder>>;
    async fn set_fav(&self, aid: u64, add: &[u64], remove: &[u64]) -> Result<()>;
    /// Adds the video to 稍后再看.
    async fn watch_later(&self, bvid: &str) -> Result<()>;
    async fn follow(&self, mid: u64, on: bool) -> Result<()>;
}

/// The user's favourite folders and what is in them. Used by `Favorites` only.
#[async_trait]
pub trait FavoritesApi: Send + Sync {
    async fn my_folders(&self) -> Result<Vec<FavFolder>>;
    /// Page `pn` (from 1) of a folder; also whether more follow.
    async fn fav_items(&self, folder_id: u64, pn: u32) -> Result<(Vec<VideoCard>, bool)>;
    /// A new folder; returns its id.
    async fn create_folder(&self, title: &str, intro: &str, private: bool) -> Result<u64>;
    async fn edit_folder(&self, id: u64, title: &str, intro: &str, private: bool) -> Result<()>;
    async fn delete_folder(&self, id: u64) -> Result<()>;
    /// A folder's 简介 (the folder list does not carry it).
    async fn folder_intro(&self, id: u64) -> Result<String>;
    /// Takes videos (by aid) out of a folder.
    async fn remove_from_folder(&self, folder: u64, aids: &[u64]) -> Result<()>;
    /// Moves (or with `keep`, copies) videos from one folder to another.
    async fn transfer(&self, from: u64, to: u64, aids: &[u64], keep: bool) -> Result<()>;
}

/// The 稍后再看 list. Used by `WatchLater` only.
#[async_trait]
pub trait WatchLaterApi: Send + Sync {
    /// With how far each was watched (`progress_s` -1: to the end) and when added.
    async fn list(&self) -> Result<Vec<HistoryEntry>>;
    /// Takes one video (by aid) out.
    async fn remove(&self, aid: u64) -> Result<()>;
    /// Takes every video already watched to the end out.
    async fn clear_watched(&self) -> Result<()>;
}

/// 历史记录: reading it, and reporting playback into it. Used by `History` only.
#[async_trait]
pub trait HistoryApi: Send + Sync {
    /// Video history; pass the returned cursor back for the next page.
    async fn history(&self, cursor: Option<&str>) -> Result<(Vec<HistoryEntry>, Option<String>)>;
    /// Playback heartbeat: puts the video in 历史记录 with its progress. `play_type` is 1 when
    /// playback starts, 0 while playing, 2 on pause / leaving. Times in seconds.
    async fn heartbeat(&self, aid: u64, cid: u64, played_s: u64, realtime_s: u64, play_type: u8) -> Result<()>;
}

/// Whom the user follows. Used by `Following` only.
#[async_trait]
pub trait FollowingsApi: Send + Sync {
    /// One page (50) of the user's followings, newest follow first; also the total.
    async fn followings(&self, pn: u32) -> Result<(Vec<Following>, u32)>;
}

/// What a 一键三连 request achieved; each part can fail on its own (e.g. coins run out).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Triple {
    pub liked: bool,
    pub coined: bool,
    pub favorited: bool,
    /// Coins given by this request.
    pub coins: u32,
}

pub struct WebAccount {
    http: Arc<HttpClient>,
}

impl WebAccount {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }

    fn my_mid(&self) -> Result<u64> {
        self.http.session().dede_user_id.and_then(|m| m.parse().ok()).ok_or(ApiError::NotLoggedIn)
    }
}

/// `aid:2,aid:2`: videos as the folder endpoints name them (type 2 is a video).
fn video_resources(aids: &[u64]) -> String {
    aids.iter().map(|a| format!("{a}:2")).collect::<Vec<_>>().join(",")
}

fn ids(list: &[u64]) -> String {
    list.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}

pub(crate) fn history_of(v: &Value) -> Option<HistoryEntry> {
    let h = &v["history"];
    if str_of(&h["business"]) != "archive" {
        return None;
    }
    Some(HistoryEntry {
        card: VideoCard {
            bvid: str_of(&h["bvid"]),
            aid: u64_of(&h["oid"]),
            cid: Some(u64_of(&h["cid"])).filter(|&c| c > 0),
            title: title_of(&v["title"]),
            cover: image_of(&v["cover"]),
            owner: Owner { mid: u64_of(&v["author_mid"]), name: str_of(&v["author_name"]) },
            duration_s: duration_of(&v["duration"]),
            raw: v.clone(),
            ..Default::default()
        },
        progress_s: v["progress"].as_i64().unwrap_or(0),
        viewed_at: u64_of(&v["view_at"]) as i64,
    })
}

/// A 稍后再看 item: the video, how far it was watched and when it was added.
pub(crate) fn watch_later_of(v: &Value) -> HistoryEntry {
    let duration = u64_of(&v["duration"]) as i64;
    let progress = v["progress"].as_i64().unwrap_or(0);
    HistoryEntry {
        card: card_of(v, SourceKind::Related),
        // Watched to (nearly) the end counts as finished, as B站 marks it.
        progress_s: if progress < 0 || (duration > 0 && progress >= duration - 1) { -1 } else { progress },
        viewed_at: u64_of(&v["add_at"]) as i64,
    }
}

pub(crate) fn triple_of(d: &Value) -> Triple {
    let coined = d["coin"].as_bool().unwrap_or(false);
    Triple {
        liked: d["like"].as_bool().unwrap_or(false),
        coined,
        favorited: d["fav"].as_bool().unwrap_or(false),
        coins: if coined { u64_of(&d["multiply"]) as u32 } else { 0 },
    }
}

pub(crate) fn fav_item_of(v: &Value) -> VideoCard {
    let cnt = &v["cnt_info"];
    VideoCard {
        bvid: str_of(&v["bvid"]),
        aid: u64_of(&v["id"]),
        title: title_of(&v["title"]),
        desc: str_of(&v["intro"]),
        cover: image_of(&v["cover"]),
        owner: Owner { mid: u64_of(&v["upper"]["mid"]), name: str_of(&v["upper"]["name"]) },
        duration_s: duration_of(&v["duration"]),
        pubdate: u64_of(&v["pubtime"]) as i64,
        stats: Stats {
            view: u64_of(&cnt["play"]),
            danmaku: u64_of(&cnt["danmaku"]),
            favorite: u64_of(&cnt["collect"]),
            ..Default::default()
        },
        raw: v.clone(),
        ..Default::default()
    }
}

impl WebAccount {
    /// The user's own folders; with `aid > 0` each is marked if it holds that video.
    async fn folders(&self, aid: u64) -> Result<Vec<FavFolder>> {
        let mid = self.my_mid()?;
        let mut p = params!["up_mid" => mid, "type" => 2];
        if aid > 0 {
            p.push(("rid".into(), aid.to_string()));
        }
        let d = self.http.get_json(FAV_FOLDERS, p, false).await?;
        Ok(d["list"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|f| FavFolder {
                        id: u64_of(&f["id"]),
                        title: str_of(&f["title"]),
                        count: u64_of(&f["media_count"]) as u32,
                        contains: u64_of(&f["fav_state"]) == 1,
                        private: folder_private(&f["attr"]),
                        default: folder_default(&f["attr"]),
                    })
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[async_trait]
impl VideoActionApi for WebAccount {
    async fn relation(&self, bvid: &str) -> Result<VideoRelation> {
        let d = self.http.get_json(ARCHIVE_RELATION, params!["bvid" => bvid], false).await?;
        Ok(VideoRelation {
            liked: d["like"].as_bool().unwrap_or(false),
            coins: u64_of(&d["coin"]) as u32,
            favorited: d["favorite"].as_bool().unwrap_or(false),
            following_up: d["attention"].as_bool().unwrap_or(false),
        })
    }

    async fn like(&self, bvid: &str, on: bool) -> Result<()> {
        self.http.post_form(LIKE, params!["bvid" => bvid, "like" => if on { 1 } else { 2 }]).await.map(|_| ())
    }

    async fn coin(&self, bvid: &str, count: u32, also_like: bool) -> Result<()> {
        let form = params!["bvid" => bvid, "multiply" => count.clamp(1, 2), "select_like" => u8::from(also_like)];
        self.http.post_form(COIN, form).await.map(|_| ())
    }

    async fn triple(&self, bvid: &str) -> Result<Triple> {
        self.http.post_form(TRIPLE, params!["bvid" => bvid]).await.map(|d| triple_of(&d))
    }

    async fn fav_folders(&self, aid: u64) -> Result<Vec<FavFolder>> {
        self.folders(aid).await
    }

    async fn set_fav(&self, aid: u64, add: &[u64], remove: &[u64]) -> Result<()> {
        let form = params!["rid" => aid, "type" => 2, "add_media_ids" => ids(add), "del_media_ids" => ids(remove)];
        self.http.post_form(FAV_DEAL, form).await.map(|_| ())
    }

    async fn watch_later(&self, bvid: &str) -> Result<()> {
        self.http.post_form(TOVIEW_ADD, params!["bvid" => bvid]).await.map(|_| ())
    }

    async fn follow(&self, mid: u64, on: bool) -> Result<()> {
        let form = params!["fid" => mid, "act" => if on { 1 } else { 2 }, "re_src" => 11];
        self.http.post_form(RELATION_MODIFY, form).await.map(|_| ())
    }
}

#[async_trait]
impl FavoritesApi for WebAccount {
    async fn my_folders(&self) -> Result<Vec<FavFolder>> {
        self.folders(0).await
    }

    async fn fav_items(&self, folder_id: u64, pn: u32) -> Result<(Vec<VideoCard>, bool)> {
        let d = self
            .http
            .get_json(FAV_LIST, params!["media_id" => folder_id, "pn" => pn, "ps" => 20, "platform" => "web"], false)
            .await?;
        let items = d["medias"].as_array().map(|a| a.iter().map(fav_item_of).collect()).unwrap_or_default();
        Ok((items, d["has_more"].as_bool().unwrap_or(false)))
    }

    async fn create_folder(&self, title: &str, intro: &str, private: bool) -> Result<u64> {
        let form = params!["title" => title, "intro" => intro, "privacy" => u8::from(private), "cover" => ""];
        self.http.post_form(FOLDER_ADD, form).await.map(|d| u64_of(&d["id"]))
    }

    async fn edit_folder(&self, id: u64, title: &str, intro: &str, private: bool) -> Result<()> {
        let form = params!["media_id" => id, "title" => title, "intro" => intro, "privacy" => u8::from(private), "cover" => ""];
        self.http.post_form(FOLDER_EDIT, form).await.map(|_| ())
    }

    async fn delete_folder(&self, id: u64) -> Result<()> {
        self.http.post_form(FOLDER_DEL, params!["media_ids" => id]).await.map(|_| ())
    }

    async fn folder_intro(&self, id: u64) -> Result<String> {
        let d = self.http.get_json(FOLDER_INFO, params!["media_id" => id], false).await?;
        Ok(str_of(&d["intro"]))
    }

    async fn remove_from_folder(&self, folder: u64, aids: &[u64]) -> Result<()> {
        let form = params!["media_id" => folder, "resources" => video_resources(aids), "platform" => "web"];
        self.http.post_form(FAV_BATCH_DEL, form).await.map(|_| ())
    }

    async fn transfer(&self, from: u64, to: u64, aids: &[u64], keep: bool) -> Result<()> {
        let form = params![
            "src_media_id" => from, "tar_media_id" => to, "mid" => self.my_mid()?,
            "resources" => video_resources(aids), "platform" => "web",
        ];
        self.http.post_form(if keep { FAV_COPY } else { FAV_MOVE }, form).await.map(|_| ())
    }
}

#[async_trait]
impl WatchLaterApi for WebAccount {
    async fn list(&self) -> Result<Vec<HistoryEntry>> {
        let d = self.http.get_json(TOVIEW_LIST, params![], false).await?;
        Ok(d["list"].as_array().map(|a| a.iter().map(watch_later_of).collect()).unwrap_or_default())
    }

    async fn remove(&self, aid: u64) -> Result<()> {
        self.http.post_form(TOVIEW_DEL, params!["aid" => aid]).await.map(|_| ())
    }

    async fn clear_watched(&self) -> Result<()> {
        self.http.post_form(TOVIEW_DEL, params!["viewed" => "true"]).await.map(|_| ())
    }
}

#[async_trait]
impl HistoryApi for WebAccount {
    async fn history(&self, cursor: Option<&str>) -> Result<(Vec<HistoryEntry>, Option<String>)> {
        let mut p = params!["ps" => 20, "type" => "archive"];
        if let Some((max, view_at)) = cursor.and_then(|c| c.split_once(':')) {
            p.push(("max".into(), max.into()));
            p.push(("view_at".into(), view_at.into()));
            p.push(("business".into(), "archive".into()));
        }
        let d = self.http.get_json(HISTORY, p, false).await?;
        let list: Vec<HistoryEntry> =
            d["list"].as_array().map(|a| a.iter().filter_map(history_of).collect()).unwrap_or_default();
        let c = &d["cursor"];
        let next = (!list.is_empty() && u64_of(&c["max"]) > 0)
            .then(|| format!("{}:{}", u64_of(&c["max"]), u64_of(&c["view_at"])));
        Ok((list, next))
    }

    async fn heartbeat(&self, aid: u64, cid: u64, played_s: u64, realtime_s: u64, play_type: u8) -> Result<()> {
        let mid = self.my_mid()?;
        let form = params![
            "aid" => aid, "cid" => cid, "mid" => mid, "played_time" => played_s, "realtime" => realtime_s,
            "start_ts" => rubi_model::now_secs() - realtime_s as i64, "type" => 3, "dt" => 2, "play_type" => play_type,
        ];
        self.http.post_form(HEARTBEAT, form).await.map(|_| ())
    }
}

#[async_trait]
impl FollowingsApi for WebAccount {
    async fn followings(&self, pn: u32) -> Result<(Vec<Following>, u32)> {
        let mid = self.my_mid()?;
        let d = self.http.get_json(FOLLOWINGS, params!["vmid" => mid, "pn" => pn, "ps" => 50], false).await?;
        let list = d["list"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|u| Following {
                        mid: u64_of(&u["mid"]),
                        name: str_of(&u["uname"]),
                        face: image_of(&u["face"]),
                        followed_at: u64_of(&u["mtime"]) as i64,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok((list, u64_of(&d["total"]) as u32))
    }
}

/// Folder `attr` bit 0: private.
fn folder_private(attr: &Value) -> bool {
    u64_of(attr) & 1 == 1
}

/// Folder `attr` bit 1 clear: the default folder.
fn folder_default(attr: &Value) -> bool {
    u64_of(attr) & 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_videos_for_folders() {
        assert_eq!(video_resources(&[1, 23]), "1:2,23:2");
    }

    #[test]
    fn reads_folder_attr() {
        use serde_json::json;
        assert!(folder_default(&json!(0)) && !folder_private(&json!(0)));
        assert!(folder_default(&json!(1)) && folder_private(&json!(1)));
        assert!(!folder_default(&json!(2)) && !folder_private(&json!(2)));
        assert!(!folder_default(&json!(3)) && folder_private(&json!(3)));
    }
    use serde_json::json;

    #[test]
    fn parses_history_and_skips_non_videos() {
        let v = json!({"title": "t", "author_name": "up", "author_mid": 3, "progress": -1, "view_at": 9, "duration": 100,
            "history": {"business": "archive", "bvid": "BV1", "oid": 7, "cid": 8}});
        let h = history_of(&v).unwrap();
        assert_eq!((h.card.aid, h.card.cid, h.progress_s, h.viewed_at), (7, Some(8), -1, 9));
        assert!(history_of(&json!({"history": {"business": "live"}})).is_none());
    }

    #[test]
    fn parses_triple() {
        let t = triple_of(&json!({"like": true, "coin": false, "fav": true, "multiply": 2}));
        assert_eq!(t, Triple { liked: true, coined: false, favorited: true, coins: 0 });
        assert_eq!(triple_of(&json!({"like": true, "coin": true, "fav": true, "multiply": 1})).coins, 1);
    }

    #[test]
    fn parses_fav_item() {
        let v = json!({"id": 5, "bvid": "BV5", "title": "t", "upper": {"mid": 1, "name": "u"}, "cnt_info": {"play": 10, "collect": 2}});
        let c = fav_item_of(&v);
        assert_eq!((c.aid, c.stats.view, c.stats.favorite), (5, 10, 2));
    }
}
