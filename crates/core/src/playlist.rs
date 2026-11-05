//! The custom play lists: videos added from anywhere, kept in the local database and part of
//! the export. [`Playlists`] edits them; playing one is [`crate::PlayQueue`]'s, hearing one
//! [`crate::Listening`]'s, both over the shared [`crate::lineup::Lineup`].

use std::sync::Arc;

use rubi_model::VideoCard;
use rubi_store::{DEFAULT_PLAYLIST, KvRepo, PlaylistInfo, PlaylistRepo, SqliteStore};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::lineup::Lineup;
use crate::ports::{PlaylistBackup, VideoLookup};

const MODE_KEY: &str = "playlist_mode";
/// The name of the list that always exists.
pub const DEFAULT_NAME: &str = "默认";
/// A 收藏夹 or 稍后再看 played through: kept in memory only, dropped when its video page closes.
pub const TEMP_PLAYLIST: i64 = -1;
/// 听视频 of one video, followed by its 相关推荐 as they come: in memory only.
pub const RELATED_PLAYLIST: i64 = -2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaylistMode {
    /// Front to back, then stop.
    #[default]
    Sequential,
    /// Front to back, then from the front again.
    LoopAll,
    /// The same video again.
    LoopOne,
    /// Any other video at random.
    Shuffle,
}

/// Longest play list name, in characters.
pub const MAX_PLAYLIST_NAME: usize = 20;

/// The stored lists and the mode they play in.
pub(crate) struct Playlist<S> {
    store: Arc<S>,
}

impl<S: PlaylistRepo + KvRepo> Playlist<S> {
    pub(crate) fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    pub fn lists(&self) -> Result<Vec<PlaylistInfo>> {
        Ok(self.store.playlists()?)
    }

    /// A new empty list; the name is trimmed and must be new.
    pub fn create(&self, name: &str) -> Result<i64> {
        let name = self.check_name(name, None)?;
        Ok(self.store.create_playlist(&name)?)
    }

    pub fn rename(&self, id: i64, name: &str) -> Result<()> {
        let name = self.check_name(name, Some(id))?;
        Ok(self.store.rename_playlist(id, &name)?)
    }

    /// Deletes a list with its videos; 默认 cannot be deleted.
    pub fn delete(&self, id: i64) -> Result<()> {
        if id == DEFAULT_PLAYLIST {
            return Err(CoreError::Invalid("默认列表不能删除".into()));
        }
        Ok(self.store.delete_playlist(id)?)
    }

    fn check_name(&self, name: &str, except: Option<i64>) -> Result<String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::Invalid("请输入列表名称".into()));
        }
        if name.chars().count() > MAX_PLAYLIST_NAME {
            return Err(CoreError::Invalid(format!("名称最多 {MAX_PLAYLIST_NAME} 个字")));
        }
        if self.lists()?.iter().any(|l| l.name == name && Some(l.id) != except) {
            return Err(CoreError::Invalid("已有同名列表".into()));
        }
        Ok(name.to_owned())
    }

    pub fn items(&self, list: i64) -> Result<Vec<VideoCard>> {
        Ok(self.store.playlist(list)?)
    }

    /// The lists that hold `bvid`.
    pub fn lists_with(&self, bvid: &str) -> Result<Vec<i64>> {
        Ok(self.store.lists_with(bvid)?)
    }

    /// Appends `card` to `list` unless it is already there; returns whether it was added.
    pub fn add(&self, list: i64, card: VideoCard) -> Result<bool> {
        Ok(self.store.add_to_playlist(list, &card)?)
    }

    pub fn remove(&self, list: i64, bvid: &str) -> Result<()> {
        Ok(self.store.remove_from_playlist(list, bvid)?)
    }

    /// Moves the item at `from` to position `to` (both clamped to the list).
    pub fn move_item(&self, list: i64, from: usize, to: usize) -> Result<()> {
        Ok(self.store.move_in_playlist(list, from, to)?)
    }

    pub fn clear(&self, list: i64) -> Result<()> {
        Ok(self.store.set_playlist(list, &[])?)
    }

    /// Replaces every list with `lists` (an imported backup): 默认 takes the one named so (or
    /// the first), the others are made anew.
    pub fn replace_all(&self, lists: &[(String, Vec<VideoCard>)]) -> Result<()> {
        let default_at = lists.iter().position(|(n, _)| n == DEFAULT_NAME).unwrap_or(0);
        Ok(self.store.replace_playlists(lists, default_at)?)
    }

    pub fn mode(&self) -> Result<PlaylistMode> {
        Ok(self.store.get_json(MODE_KEY)?.unwrap_or_default())
    }

    pub fn set_mode(&self, mode: PlaylistMode) -> Result<()> {
        Ok(self.store.put_json(MODE_KEY, &mode)?)
    }
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> PlaylistBackup for Playlist<S> {
    fn export_lists(&self) -> Result<Vec<(String, Vec<VideoCard>)>> {
        self.lists()?.into_iter().map(|l| Ok((l.name, self.items(l.id)?))).collect()
    }

    fn replace_all(&self, lists: &[(String, Vec<VideoCard>)]) -> Result<()> {
        Playlist::replace_all(self, lists)
    }
}

/// The item before `current` in list order, wrapping round.
pub(crate) fn previous_in<'a>(items: &'a [VideoCard], current: &str) -> Option<&'a VideoCard> {
    let Some(at) = items.iter().position(|c| c.bvid == current) else { return items.first() };
    items.get((at + items.len() - 1) % items.len())
}

/// The pick, with the randomness passed in so it can be tested.
pub(crate) fn next_in<'a>(
    items: &'a [VideoCard],
    current: &str,
    mode: PlaylistMode,
    random: impl Fn(usize) -> usize,
) -> Option<&'a VideoCard> {
    let Some(at) = items.iter().position(|c| c.bvid == current) else { return items.first() };
    match mode {
        PlaylistMode::Sequential => items.get(at + 1),
        PlaylistMode::LoopAll => items.get((at + 1) % items.len()),
        PlaylistMode::LoopOne => items.get(at),
        PlaylistMode::Shuffle if items.len() > 1 => {
            // Any other item: draw from the rest, then step over the current one.
            let r = random(items.len() - 1);
            items.get(if r >= at { r + 1 } else { r })
        }
        PlaylistMode::Shuffle => items.get(at),
    }
}

/// Editing the play lists: names, videos, order, and the mode they play in.
pub struct Playlists<S = SqliteStore> {
    list: Arc<Playlist<S>>,
    lineup: Arc<Lineup<S>>,
    videos: Arc<dyn VideoLookup>,
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> Playlists<S> {
    pub(crate) fn new(list: Arc<Playlist<S>>, lineup: Arc<Lineup<S>>, videos: Arc<dyn VideoLookup>) -> Self {
        Self { list, lineup, videos }
    }

    pub fn lists(&self) -> Result<Vec<PlaylistInfo>> {
        self.list.lists()
    }

    pub fn create(&self, name: &str) -> Result<i64> {
        self.list.create(name)
    }

    pub fn rename(&self, id: i64, name: &str) -> Result<()> {
        self.list.rename(id, name)
    }

    /// Deletes a stored list, or forgets an in-memory one (negative id).
    pub fn delete(&self, id: i64) -> Result<()> {
        if id >= 0 {
            self.list.delete(id)?;
        }
        self.lineup.forget(id);
        Ok(())
    }

    pub fn items(&self, list: i64) -> Result<Vec<VideoCard>> {
        self.lineup.items(list)
    }

    pub fn lists_with(&self, bvid: &str) -> Result<Vec<i64>> {
        self.list.lists_with(bvid)
    }

    /// Adds a video (looked up when it was not shown); false when it was already listed.
    pub async fn add(&self, list: i64, bvid: &str) -> Result<bool> {
        let card = self.videos.card(bvid).await?;
        self.list.add(list, card)
    }

    pub fn remove(&self, list: i64, bvid: &str) -> Result<()> {
        self.list.remove(list, bvid)
    }

    pub fn move_item(&self, list: i64, from: usize, to: usize) -> Result<()> {
        self.list.move_item(list, from, to)
    }

    pub fn clear(&self, list: i64) -> Result<()> {
        self.list.clear(list)
    }

    pub fn mode(&self) -> Result<PlaylistMode> {
        self.list.mode()
    }

    pub fn set_mode(&self, mode: PlaylistMode) -> Result<()> {
        self.list.set_mode(mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubi_store::SqliteStore;

    fn card(b: &str) -> VideoCard {
        VideoCard { bvid: b.into(), ..Default::default() }
    }

    #[test]
    fn next_by_mode() {
        let items = vec![card("a"), card("b"), card("c")];
        let pick = |cur, mode, r: usize| next_in(&items, cur, mode, move |_| r).map(|c| c.bvid.as_str());
        assert_eq!(pick("a", PlaylistMode::Sequential, 0), Some("b"));
        assert_eq!(pick("c", PlaylistMode::Sequential, 0), None);
        assert_eq!(pick("c", PlaylistMode::LoopAll, 0), Some("a"));
        assert_eq!(pick("b", PlaylistMode::LoopOne, 0), Some("b"));
        assert_eq!(pick("b", PlaylistMode::Shuffle, 0), Some("a"));
        assert_eq!(pick("b", PlaylistMode::Shuffle, 1), Some("c"), "the current one is stepped over");
        assert_eq!(pick("zzz", PlaylistMode::Sequential, 0), Some("a"));
    }

    #[test]
    fn previous_wraps_round() {
        let items = vec![card("a"), card("b"), card("c")];
        let prev = |cur| previous_in(&items, cur).map(|c| c.bvid.as_str());
        assert_eq!(prev("b"), Some("a"));
        assert_eq!(prev("a"), Some("c"));
        assert_eq!(prev("zzz"), Some("a"));
        assert_eq!(previous_in(&[], "a"), None);
    }

    #[test]
    fn stored_list_keeps_order_and_mode() {
        let p = Playlist::new(Arc::new(SqliteStore::in_memory().unwrap()));
        let d = DEFAULT_PLAYLIST;
        assert!(p.add(d, card("a")).unwrap());
        assert!(p.add(d, card("b")).unwrap());
        assert!(!p.add(d, card("a")).unwrap(), "no duplicates");
        p.add(d, card("c")).unwrap();
        p.move_item(d, 2, 0).unwrap();
        let ids: Vec<String> = p.items(d).unwrap().into_iter().map(|c| c.bvid).collect();
        assert_eq!(ids, ["c", "a", "b"]);
        p.remove(d, "a").unwrap();
        assert!(p.lists_with("a").unwrap().is_empty());
        p.set_mode(PlaylistMode::LoopAll).unwrap();
        assert_eq!(p.mode().unwrap(), PlaylistMode::LoopAll);
        assert_eq!(next_in(&p.items(d).unwrap(), "b", p.mode().unwrap(), |_| 0).unwrap().bvid, "c");
    }

    #[test]
    fn named_lists() {
        let p = Playlist::new(Arc::new(SqliteStore::in_memory().unwrap()));
        let commute = p.create("  通勤听 ").unwrap();
        assert!(matches!(p.create("通勤听"), Err(CoreError::Invalid(_))), "names are unique");
        assert!(matches!(p.create("   "), Err(CoreError::Invalid(_))));
        assert!(matches!(p.create(&"长".repeat(21)), Err(CoreError::Invalid(_))));
        p.add(commute, card("a")).unwrap();
        p.add(DEFAULT_PLAYLIST, card("a")).unwrap();
        assert_eq!(p.lists_with("a").unwrap(), vec![DEFAULT_PLAYLIST, commute]);
        p.rename(commute, "通勤").unwrap();
        p.rename(commute, "通勤").unwrap();
        let lists = p.lists().unwrap();
        assert_eq!(lists.iter().map(|l| (l.name.as_str(), l.count)).collect::<Vec<_>>(), [("默认", 1), ("通勤", 1)]);
        assert!(matches!(p.delete(DEFAULT_PLAYLIST), Err(CoreError::Invalid(_))));
        p.delete(commute).unwrap();
        assert_eq!(p.lists_with("a").unwrap(), vec![DEFAULT_PLAYLIST]);
        p.replace_all(&[("默认".into(), vec![card("x")]), ("学习".into(), vec![card("y"), card("z")])]).unwrap();
        let lists = p.lists().unwrap();
        assert_eq!(lists.iter().map(|l| (l.name.as_str(), l.count)).collect::<Vec<_>>(), [("默认", 1), ("学习", 2)]);
    }
}
