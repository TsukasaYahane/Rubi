//! The custom play list: videos added from anywhere, played in order, looped or shuffled, as
//! video or as audio only (听视频). Kept in the local database and part of the export.

use std::sync::{Arc, Mutex};

use rubi_model::{VideoCard, now_secs};
use rubi_store::{KvRepo, PlaylistRepo, SqliteStore};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::queue::ListenQueue;
use crate::videos::{CardCache, Videos};

const MODE_KEY: &str = "playlist_mode";

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

/// The stored list and its mode.
pub(crate) struct Playlist<S> {
    store: Arc<S>,
}

impl<S: PlaylistRepo + KvRepo> Playlist<S> {
    pub(crate) fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    pub fn items(&self) -> Result<Vec<VideoCard>> {
        Ok(self.store.playlist()?)
    }

    pub fn contains(&self, bvid: &str) -> Result<bool> {
        Ok(self.items()?.iter().any(|c| c.bvid == bvid))
    }

    /// Appends `card` unless it is already listed; returns whether it was added.
    pub fn add(&self, card: VideoCard) -> Result<bool> {
        let mut items = self.items()?;
        if items.iter().any(|c| c.bvid == card.bvid) {
            return Ok(false);
        }
        items.push(card);
        self.store.set_playlist(&items)?;
        Ok(true)
    }

    pub fn remove(&self, bvid: &str) -> Result<()> {
        let mut items = self.items()?;
        items.retain(|c| c.bvid != bvid);
        Ok(self.store.set_playlist(&items)?)
    }

    /// Moves the item at `from` to position `to` (both clamped to the list).
    pub fn move_item(&self, from: usize, to: usize) -> Result<()> {
        let mut items = self.items()?;
        if from >= items.len() {
            return Ok(());
        }
        let card = items.remove(from);
        items.insert(to.min(items.len()), card);
        Ok(self.store.set_playlist(&items)?)
    }

    pub fn clear(&self) -> Result<()> {
        Ok(self.store.set_playlist(&[])?)
    }

    pub fn replace(&self, items: &[VideoCard]) -> Result<()> {
        Ok(self.store.set_playlist(items)?)
    }

    pub fn mode(&self) -> Result<PlaylistMode> {
        Ok(self.store.get_json(MODE_KEY)?.unwrap_or_default())
    }

    pub fn set_mode(&self, mode: PlaylistMode) -> Result<()> {
        Ok(self.store.put_json(MODE_KEY, &mode)?)
    }

    /// Where 播放全部 starts: the first item, or a random one when shuffling.
    pub fn first(&self) -> Result<Option<VideoCard>> {
        let items = self.items()?;
        Ok(match self.mode()? {
            PlaylistMode::Shuffle if !items.is_empty() => Some(items[random_below(items.len())].clone()),
            _ => items.into_iter().next(),
        })
    }

    /// What plays after `current` ends; `None` stops. A video not in the list is followed by
    /// the first item.
    pub fn next_after(&self, current: &str) -> Result<Option<VideoCard>> {
        let items = self.items()?;
        Ok(next_in(&items, current, self.mode()?, random_below).cloned())
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

/// The play list as the app uses it: editing it, playing it as video, and hearing it (听视频).
pub struct Playlists {
    list: Arc<Playlist<SqliteStore>>,
    videos: Arc<Videos>,
    cards: Arc<CardCache>,
    listening: Mutex<ListenQueue>,
}

impl Playlists {
    pub(crate) fn new(list: Arc<Playlist<SqliteStore>>, videos: Arc<Videos>, cards: Arc<CardCache>) -> Self {
        Self { list, videos, cards, listening: Mutex::default() }
    }

    pub fn items(&self) -> Result<Vec<VideoCard>> {
        let items = self.list.items()?;
        self.cards.remember(&items);
        Ok(items)
    }

    pub fn contains(&self, bvid: &str) -> Result<bool> {
        self.list.contains(bvid)
    }

    /// Adds a video (looked up when it was not shown); false when it was already listed.
    pub async fn add(&self, bvid: &str) -> Result<bool> {
        let card = self.videos.card(bvid).await?;
        self.list.add(card)
    }

    pub fn remove(&self, bvid: &str) -> Result<()> {
        self.list.remove(bvid)
    }

    pub fn move_item(&self, from: usize, to: usize) -> Result<()> {
        self.list.move_item(from, to)
    }

    pub fn clear(&self) -> Result<()> {
        self.list.clear()
    }

    pub fn mode(&self) -> Result<PlaylistMode> {
        self.list.mode()
    }

    pub fn set_mode(&self, mode: PlaylistMode) -> Result<()> {
        self.list.set_mode(mode)
    }

    /// Where 播放全部 / 听全部 starts.
    pub fn first(&self) -> Result<Option<VideoCard>> {
        self.list.first()
    }

    /// The video to play after `current` ended on the video page; `None` stops.
    pub fn next_after(&self, current: &str) -> Result<Option<VideoCard>> {
        self.list.next_after(current)
    }

    // ----- 听视频 -----

    /// What is being heard; `None` when not listening.
    pub fn listening(&self) -> Option<VideoCard> {
        self.queue().current()
    }

    /// Starts hearing the list from `bvid`.
    pub async fn listen_from(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.videos.card(bvid).await?;
        Ok(self.queue().start(card))
    }

    /// Switches to `bvid` picked from the list.
    pub async fn listen_jump(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.videos.card(bvid).await?;
        Ok(self.queue().jump(card))
    }

    /// 下一首 (`by_user`) or the item having ended; `None` means listening stopped.
    pub fn listen_next(&self, by_user: bool) -> Result<Option<VideoCard>> {
        let (items, mode) = (self.list.items()?, self.list.mode()?);
        Ok(self.queue().next(&items, mode, by_user, random_below))
    }

    /// 上一首.
    pub fn listen_previous(&self) -> Result<Option<VideoCard>> {
        let (items, mode) = (self.list.items()?, self.list.mode()?);
        Ok(self.queue().previous(&items, mode))
    }

    pub fn listen_stop(&self) {
        self.queue().stop();
    }

    fn queue(&self) -> std::sync::MutexGuard<'_, ListenQueue> {
        self.listening.lock().unwrap_or_else(|e| e.into_inner())
    }
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
        assert!(p.add(card("a")).unwrap());
        assert!(p.add(card("b")).unwrap());
        assert!(!p.add(card("a")).unwrap(), "no duplicates");
        p.add(card("c")).unwrap();
        p.move_item(2, 0).unwrap();
        let ids: Vec<String> = p.items().unwrap().into_iter().map(|c| c.bvid).collect();
        assert_eq!(ids, ["c", "a", "b"]);
        p.remove("a").unwrap();
        assert!(!p.contains("a").unwrap());
        p.set_mode(PlaylistMode::LoopAll).unwrap();
        assert_eq!(p.mode().unwrap(), PlaylistMode::LoopAll);
        assert_eq!(p.next_after("b").unwrap().unwrap().bvid, "c");
    }
}
