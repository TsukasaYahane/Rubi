//! The custom play list: videos added from anywhere, played in order, looped or shuffled, as
//! video or as audio only (听视频). Kept in the local database and part of the export.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use futures_util::future::join_all;
use rubi_model::{VideoCard, now_secs};
use rubi_store::{DEFAULT_PLAYLIST, KvRepo, PlaylistInfo, PlaylistRepo, SqliteStore};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
use crate::ports::{PlaylistBackup, VideoLookup};
use crate::queue::ListenQueue;
use crate::videos::CardCache;

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

/// A list that lives in memory only (negative ids).
struct Adhoc {
    name: String,
    items: Vec<VideoCard>,
}

/// The play lists as the app uses them: editing them, playing one as video, and hearing one
/// (听视频). The list last started is the one whose next item follows.
pub struct Playlists<S = SqliteStore> {
    list: Arc<Playlist<S>>,
    videos: Arc<dyn VideoLookup>,
    cards: Arc<CardCache>,
    listening: Mutex<ListenQueue>,
    active: Mutex<i64>,
    adhoc: Mutex<HashMap<i64, Adhoc>>,
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> Playlists<S> {
    pub(crate) fn new(list: Arc<Playlist<S>>, videos: Arc<dyn VideoLookup>, cards: Arc<CardCache>) -> Self {
        Self {
            list,
            videos,
            cards,
            listening: Mutex::default(),
            active: Mutex::new(DEFAULT_PLAYLIST),
            adhoc: Mutex::default(),
        }
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

    pub fn delete(&self, id: i64) -> Result<()> {
        if id < 0 {
            self.drop_adhoc(id);
            return Ok(());
        }
        self.list.delete(id)?;
        let mut active = self.active.lock().unwrap_or_else(|e| e.into_inner());
        if *active == id {
            *active = DEFAULT_PLAYLIST;
        }
        Ok(())
    }

    pub fn items(&self, list: i64) -> Result<Vec<VideoCard>> {
        let items = if list < 0 {
            self.adhoc_lock().get(&list).map(|a| a.items.clone()).unwrap_or_default()
        } else {
            self.list.items(list)?
        };
        self.cards.remember(&items);
        Ok(items)
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

    /// Makes `list` the one played: its next item follows the video that ends.
    pub fn play(&self, list: i64) {
        *self.active.lock().unwrap_or_else(|e| e.into_inner()) = list;
    }

    /// Where 播放全部 / 听全部 starts; `list` becomes the one played.
    pub fn first(&self, list: i64) -> Result<Option<VideoCard>> {
        self.play(list);
        let items = self.items(list)?;
        Ok(match self.mode_of(list)? {
            PlaylistMode::Shuffle if !items.is_empty() => Some(items[random_below(items.len())].clone()),
            _ => items.into_iter().next(),
        })
    }

    /// The video to play after `current` ended on the video page; `None` stops.
    pub fn next_after(&self, current: &str) -> Result<Option<VideoCard>> {
        let list = self.active();
        let items = self.items(list)?;
        Ok(next_in(&items, current, self.mode_of(list)?, random_below).cloned())
    }

    /// The list being played: its id, name and videos.
    pub fn playing(&self) -> Result<(i64, String, Vec<VideoCard>)> {
        let list = self.active();
        let name = if list < 0 {
            self.adhoc_lock().get(&list).map(|a| a.name.clone()).unwrap_or_default()
        } else {
            self.list.lists()?.into_iter().find(|l| l.id == list).map(|l| l.name).unwrap_or_default()
        };
        Ok((list, name, self.items(list)?))
    }

    /// Plays `cards` (a 收藏夹, 稍后再看) as a list of their own that is not kept: it replaces
    /// the last such list and goes with [Self::drop_temp]. Returns the first to play.
    pub async fn play_temp(&self, name: &str, bvids: &[String]) -> Result<Option<VideoCard>> {
        let cards = self.cards_of(bvids).await;
        self.adhoc_lock().insert(TEMP_PLAYLIST, Adhoc { name: name.to_owned(), items: dedup(cards) });
        self.first(TEMP_PLAYLIST)
    }

    /// Adds later pages of the list being played through.
    pub async fn extend_temp(&self, bvids: &[String]) {
        let cards = self.cards_of(bvids).await;
        self.extend(TEMP_PLAYLIST, cards);
    }

    /// The cards of `bvids` (already shown, so mostly cached; the rest are looked up together);
    /// ones that cannot be found are left out.
    async fn cards_of(&self, bvids: &[String]) -> Vec<VideoCard> {
        join_all(bvids.iter().map(|b| self.videos.card(b))).await.into_iter().filter_map(|c| c.ok()).collect()
    }

    /// The video page playing the temporary list closed: the list goes.
    pub fn drop_temp(&self) {
        self.drop_adhoc(TEMP_PLAYLIST);
    }

    fn extend(&self, list: i64, cards: Vec<VideoCard>) {
        if let Some(a) = self.adhoc_lock().get_mut(&list) {
            let mut have: HashSet<String> = a.items.iter().map(|i| i.bvid.clone()).collect();
            a.items.extend(cards.into_iter().filter(|c| have.insert(c.bvid.clone())));
        }
    }

    fn drop_adhoc(&self, list: i64) {
        self.adhoc_lock().remove(&list);
        let mut active = self.active.lock().unwrap_or_else(|e| e.into_inner());
        if *active == list {
            *active = DEFAULT_PLAYLIST;
        }
    }

    /// 相关推荐 plays in order; the other lists in the chosen mode.
    fn mode_of(&self, list: i64) -> Result<PlaylistMode> {
        if list == RELATED_PLAYLIST { Ok(PlaylistMode::Sequential) } else { self.list.mode() }
    }

    fn active(&self) -> i64 {
        *self.active.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn adhoc_lock(&self) -> std::sync::MutexGuard<'_, HashMap<i64, Adhoc>> {
        self.adhoc.lock().unwrap_or_else(|e| e.into_inner())
    }

    // ----- 听视频 -----

    /// What is being heard; `None` when not listening.
    pub fn listening(&self) -> Option<VideoCard> {
        self.queue().current()
    }

    /// Starts hearing `list` from `bvid`.
    pub async fn listen_from(&self, list: i64, bvid: &str) -> Result<VideoCard> {
        let card = self.videos.card(bvid).await?;
        self.play(list);
        Ok(self.queue().start(card))
    }

    /// Switches to `bvid` picked from the list.
    pub async fn listen_jump(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.videos.card(bvid).await?;
        Ok(self.queue().jump(card))
    }

    /// Starts hearing one video, to be followed by its 相关推荐 ([Self::listen_append]).
    pub async fn listen_related(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.videos.card(bvid).await?;
        self.adhoc_lock().insert(RELATED_PLAYLIST, Adhoc { name: "相关推荐".into(), items: vec![card.clone()] });
        self.play(RELATED_PLAYLIST);
        Ok(self.queue().start(card))
    }

    /// How many videos are left to hear after the current one when following 相关推荐;
    /// `None` when hearing a list of the user's own.
    pub fn related_left(&self) -> Option<usize> {
        if self.active() != RELATED_PLAYLIST {
            return None;
        }
        let now = self.queue().current()?;
        let items = self.items(RELATED_PLAYLIST).ok()?;
        Some(items.iter().position(|c| c.bvid == now.bvid).map(|at| items.len() - at - 1).unwrap_or(0))
    }

    /// Adds 相关推荐 to come after what is heard; ones already in the list are left out.
    pub async fn listen_append(&self, bvids: &[String]) {
        let cards = self.cards_of(bvids).await;
        self.extend(RELATED_PLAYLIST, cards);
    }

    /// 下一首 (`by_user`) or the item having ended; `None` means listening stopped.
    pub fn listen_next(&self, by_user: bool) -> Result<Option<VideoCard>> {
        let list = self.active();
        let (items, mode) = (self.items(list)?, self.mode_of(list)?);
        // Following 相关推荐 never wraps back to the first.
        let by_user = by_user && list != RELATED_PLAYLIST;
        Ok(self.queue().next(&items, mode, by_user, random_below))
    }

    /// 上一首.
    pub fn listen_previous(&self) -> Result<Option<VideoCard>> {
        let list = self.active();
        let (items, mode) = (self.items(list)?, self.mode_of(list)?);
        Ok(self.queue().previous(&items, mode))
    }

    /// The list being heard, for the 听视频 page.
    pub fn listening_items(&self) -> Result<Vec<VideoCard>> {
        self.items(self.active())
    }

    pub fn listen_stop(&self) {
        self.queue().stop();
        self.drop_adhoc(RELATED_PLAYLIST);
    }

    fn queue(&self) -> std::sync::MutexGuard<'_, ListenQueue> {
        self.listening.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// `cards` without repeats, first one kept.
fn dedup(cards: Vec<VideoCard>) -> Vec<VideoCard> {
    let mut seen = HashSet::with_capacity(cards.len());
    cards.into_iter().filter(|c| seen.insert(c.bvid.clone())).collect()
}

/// A number in `0..n` (n > 0). Shuffling needs no more than the clock's nanoseconds.
fn random_below(n: usize) -> usize {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let mut x = nanos ^ (now_secs() as u64).rotate_left(17) ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x % n.max(1) as u64) as usize
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
