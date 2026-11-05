//! 听视频: hearing a list (or one video and its 相关推荐) as audio, with its own 上一首 /
//! 下一首 history over the list being played.

use std::sync::{Arc, Mutex, MutexGuard};

use rubi_model::VideoCard;
use rubi_store::{KvRepo, PlaylistRepo, SqliteStore};

use crate::error::Result;
use crate::lineup::{Lineup, random_below};
use crate::playlist::RELATED_PLAYLIST;
use crate::queue::ListenQueue;

pub struct Listening<S = SqliteStore> {
    lineup: Arc<Lineup<S>>,
    queue: Mutex<ListenQueue>,
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> Listening<S> {
    pub(crate) fn new(lineup: Arc<Lineup<S>>) -> Self {
        Self { lineup, queue: Mutex::default() }
    }

    /// What is being heard; `None` when not listening.
    pub fn current(&self) -> Option<VideoCard> {
        self.queue().current()
    }

    /// Starts hearing `list` from `bvid`.
    pub async fn start(&self, list: i64, bvid: &str) -> Result<VideoCard> {
        let card = self.lineup.card(bvid).await?;
        self.lineup.set_active(list);
        Ok(self.queue().start(card))
    }

    /// Switches to `bvid` picked from the list.
    pub async fn jump(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.lineup.card(bvid).await?;
        Ok(self.queue().jump(card))
    }

    /// Starts hearing one video, to be followed by its 相关推荐 ([Self::append]).
    pub async fn start_related(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.lineup.card(bvid).await?;
        self.lineup.put(RELATED_PLAYLIST, "相关推荐", vec![card.clone()]);
        self.lineup.set_active(RELATED_PLAYLIST);
        Ok(self.queue().start(card))
    }

    /// How many videos are left to hear after the current one when following 相关推荐;
    /// `None` when hearing a list of the user's own.
    pub fn related_left(&self) -> Option<usize> {
        if self.lineup.active() != RELATED_PLAYLIST {
            return None;
        }
        let now = self.queue().current()?;
        let items = self.lineup.items(RELATED_PLAYLIST).ok()?;
        Some(items.iter().position(|c| c.bvid == now.bvid).map(|at| items.len() - at - 1).unwrap_or(0))
    }

    /// Adds 相关推荐 to come after what is heard; ones already in the list are left out.
    pub async fn append(&self, bvids: &[String]) {
        let cards = self.lineup.cards_of(bvids).await;
        self.lineup.extend(RELATED_PLAYLIST, cards);
    }

    /// 下一首 (`by_user`) or the item having ended; `None` means listening stopped.
    pub fn next(&self, by_user: bool) -> Result<Option<VideoCard>> {
        let list = self.lineup.active();
        let (items, mode) = (self.lineup.items(list)?, self.lineup.mode_of(list)?);
        // Following 相关推荐 never wraps back to the first.
        let by_user = by_user && list != RELATED_PLAYLIST;
        Ok(self.queue().next(&items, mode, by_user, random_below))
    }

    /// 上一首.
    pub fn previous(&self) -> Result<Option<VideoCard>> {
        let list = self.lineup.active();
        let (items, mode) = (self.lineup.items(list)?, self.lineup.mode_of(list)?);
        Ok(self.queue().previous(&items, mode))
    }

    /// The list being heard, for the 听视频 page.
    pub fn items(&self) -> Result<Vec<VideoCard>> {
        self.lineup.items(self.lineup.active())
    }

    pub fn stop(&self) {
        self.queue().stop();
        self.lineup.forget(RELATED_PLAYLIST);
    }

    fn queue(&self) -> MutexGuard<'_, ListenQueue> {
        self.queue.lock().unwrap_or_else(|e| e.into_inner())
    }
}
