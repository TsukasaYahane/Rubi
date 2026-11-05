//! What is being played: the list last started (stored, or one kept in memory only) and how
//! to read any list with the mode it plays in. Shared by [`crate::PlayQueue`] (the video page)
//! and [`crate::Listening`] (听视频), so the next video follows the same list either way.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use futures_util::future::join_all;
use rubi_model::{VideoCard, now_secs};
use rubi_store::{DEFAULT_PLAYLIST, KvRepo, PlaylistRepo};

use crate::error::Result;
use crate::playlist::{Playlist, PlaylistMode, RELATED_PLAYLIST};
use crate::ports::VideoLookup;
use crate::videos::CardCache;

/// A list that lives in memory only (negative ids).
struct Adhoc {
    name: String,
    items: Vec<VideoCard>,
}

pub(crate) struct Lineup<S> {
    list: Arc<Playlist<S>>,
    videos: Arc<dyn VideoLookup>,
    cards: Arc<CardCache>,
    active: Mutex<i64>,
    adhoc: Mutex<HashMap<i64, Adhoc>>,
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> Lineup<S> {
    pub(crate) fn new(list: Arc<Playlist<S>>, videos: Arc<dyn VideoLookup>, cards: Arc<CardCache>) -> Self {
        Self { list, videos, cards, active: Mutex::new(DEFAULT_PLAYLIST), adhoc: Mutex::default() }
    }

    /// A list's videos, stored or in memory.
    pub(crate) fn items(&self, list: i64) -> Result<Vec<VideoCard>> {
        let items = if list < 0 {
            self.adhoc_lock().get(&list).map(|a| a.items.clone()).unwrap_or_default()
        } else {
            self.list.items(list)?
        };
        self.cards.remember(&items);
        Ok(items)
    }

    pub(crate) fn name_of(&self, list: i64) -> Result<String> {
        Ok(if list < 0 {
            self.adhoc_lock().get(&list).map(|a| a.name.clone()).unwrap_or_default()
        } else {
            self.list.lists()?.into_iter().find(|l| l.id == list).map(|l| l.name).unwrap_or_default()
        })
    }

    /// 相关推荐 plays in order; the other lists in the chosen mode.
    pub(crate) fn mode_of(&self, list: i64) -> Result<PlaylistMode> {
        if list == RELATED_PLAYLIST { Ok(PlaylistMode::Sequential) } else { self.list.mode() }
    }

    /// The list being played.
    pub(crate) fn active(&self) -> i64 {
        *self.active.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Makes `list` the one played: its next item follows the video that ends.
    pub(crate) fn set_active(&self, list: i64) {
        *self.active.lock().unwrap_or_else(|e| e.into_inner()) = list;
    }

    /// Keeps `items` as in-memory list `id`, replacing what it held.
    pub(crate) fn put(&self, id: i64, name: &str, items: Vec<VideoCard>) {
        self.adhoc_lock().insert(id, Adhoc { name: name.to_owned(), items: dedup(items) });
    }

    /// Appends to in-memory list `id`; ones already there are left out.
    pub(crate) fn extend(&self, id: i64, cards: Vec<VideoCard>) {
        if let Some(a) = self.adhoc_lock().get_mut(&id) {
            let mut have: HashSet<String> = a.items.iter().map(|i| i.bvid.clone()).collect();
            a.items.extend(cards.into_iter().filter(|c| have.insert(c.bvid.clone())));
        }
    }

    /// The list is gone (deleted, or an in-memory one dropped): 默认 plays next if it was active.
    pub(crate) fn forget(&self, list: i64) {
        if list < 0 {
            self.adhoc_lock().remove(&list);
        }
        let mut active = self.active.lock().unwrap_or_else(|e| e.into_inner());
        if *active == list {
            *active = DEFAULT_PLAYLIST;
        }
    }

    pub(crate) async fn card(&self, bvid: &str) -> Result<VideoCard> {
        self.videos.card(bvid).await
    }

    /// The cards of `bvids` (already shown, so mostly cached; the rest are looked up together);
    /// ones that cannot be found are left out.
    pub(crate) async fn cards_of(&self, bvids: &[String]) -> Vec<VideoCard> {
        join_all(bvids.iter().map(|b| self.videos.card(b))).await.into_iter().filter_map(|c| c.ok()).collect()
    }

    fn adhoc_lock(&self) -> MutexGuard<'_, HashMap<i64, Adhoc>> {
        self.adhoc.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// `cards` without repeats, first one kept.
fn dedup(cards: Vec<VideoCard>) -> Vec<VideoCard> {
    let mut seen = HashSet::with_capacity(cards.len());
    cards.into_iter().filter(|c| seen.insert(c.bvid.clone())).collect()
}

/// A number in `0..n` (n > 0). Shuffling needs no more than the clock's nanoseconds.
pub(crate) fn random_below(n: usize) -> usize {
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
