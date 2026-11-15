//! 听视频: hearing a list (or one video and its 相关推荐) as audio, with its own 上一首 /
//! 下一首 history over the list being played.

use std::sync::{Arc, Mutex, MutexGuard};

use rubi_model::VideoCard;
use rubi_store::{KvRepo, PlaylistRepo, SqliteStore};

use crate::error::Result;
use crate::lineup::{Lineup, random_below};
use crate::playlist::{PlaylistMode, RELATED_PLAYLIST, TEMP_PLAYLIST};
use crate::ports::RelatedSource;
use crate::queue::ListenQueue;

pub struct Listening<S = SqliteStore> {
    lineup: Arc<Lineup<S>>,
    queue: Mutex<ListenQueue>,
    /// What follows the video being heard; 相关推荐 is read from here when it starts, since
    /// [Self::next] cannot reach the network.
    related: Arc<dyn RelatedSource>,
}

impl<S: PlaylistRepo + KvRepo + Send + Sync> Listening<S> {
    pub(crate) fn new(lineup: Arc<Lineup<S>>, related: Option<Arc<dyn RelatedSource>>) -> Self {
        Self { lineup, queue: Mutex::default(), related: related.unwrap_or_else(|| Arc::new(NoRelated)) }
    }

    /// The mode the queue is played in.
    pub fn mode(&self) -> Result<PlaylistMode> {
        self.lineup.mode()
    }

    /// Sets how the queue is played from here on.
    pub fn set_mode(&self, mode: PlaylistMode) -> Result<()> {
        self.lineup.set_mode(mode)
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

    /// Whether the list being played holds `bvid`: hearing it keeps that list's order rather
    /// than bringing 相关推荐 in.
    pub fn holds(&self, bvid: &str) -> bool {
        let list = self.lineup.active();
        list != RELATED_PLAYLIST
            && self.lineup.items(list).is_ok_and(|items| items.iter().any(|card| card.bvid == bvid))
    }

    /// Starts hearing `bvid` from the list already being played when that list holds it, and
    /// from 相关推荐 otherwise: a 播放列表 the user put on keeps its order when listening starts.
    pub async fn start_here(&self, bvid: &str) -> Result<VideoCard> {
        if self.holds(bvid) {
            return self.start(self.lineup.active(), bvid).await;
        }
        self.start_related(bvid).await
    }

    /// Starts hearing `bvid` out of `bvids`, the list the video page was playing under `name`:
    /// 播放全部 hands it over, and the order it was played in is kept.
    pub async fn start_listing(&self, name: &str, bvids: &[String], bvid: &str) -> Result<VideoCard> {
        let cards = self.lineup.cards_of(bvids).await;
        self.lineup.put(TEMP_PLAYLIST, name, cards);
        self.start(TEMP_PLAYLIST, bvid).await
    }

    /// Starts hearing one video, to be followed by its 相关推荐 ([Self::append]).
    pub async fn start_related(&self, bvid: &str) -> Result<VideoCard> {
        let card = self.lineup.card(bvid).await?;
        // The video itself comes first, then what follows it: they are read now, because the
        // queue is walked without touching the network.
        let mut list = vec![card.clone()];
        list.extend(self.related.related(bvid).await);
        self.lineup.put(RELATED_PLAYLIST, "相关推荐", list);
        self.lineup.set_active(RELATED_PLAYLIST);
        Ok(self.queue().start(card))
    }

    /// Reads more of 相关推荐 once the queue runs short while hearing them.
    pub async fn extend_related(&self, bvid: &str) {
        if self.lineup.active() != RELATED_PLAYLIST {
            return;
        }
        let more = self.related.related(bvid).await;
        if more.is_empty() {
            return;
        }
        let mut items = self.lineup.items(RELATED_PLAYLIST).unwrap_or_default();
        items.extend(more);
        self.lineup.put(RELATED_PLAYLIST, "相关推荐", items);
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
        let (items, mode) = (self.lineup.items(self.lineup.active())?, self.lineup.mode()?);
        Ok(self.queue().next(&items, mode, by_user, random_below))
    }

    /// 上一首.
    pub fn previous(&self) -> Result<Option<VideoCard>> {
        let (items, mode) = (self.lineup.items(self.lineup.active())?, self.lineup.mode()?);
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

/// Nothing follows: what a queue hears when no source of 相关推荐 is given.
pub(crate) struct NoRelated;

#[async_trait::async_trait]
impl RelatedSource for NoRelated {
    async fn related(&self, _bvid: &str) -> Vec<VideoCard> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::lineup::Lineup;
    use crate::playlist::Playlist;
    use crate::ports::VideoLookup;
    use crate::videos::CardCache;

    struct Cards;

    #[async_trait]
    impl VideoLookup for Cards {
        async fn card(&self, bvid: &str) -> Result<VideoCard> {
            Ok(VideoCard { bvid: bvid.into(), ..Default::default() })
        }
    }

    fn lineup() -> Arc<Lineup<SqliteStore>> {
        let list = Arc::new(Playlist::new(Arc::new(SqliteStore::in_memory().unwrap())));
        Arc::new(Lineup::new(list, Arc::new(Cards), Arc::new(CardCache::default())))
    }

    fn cards(bvids: &[&str]) -> Vec<VideoCard> {
        bvids.iter().map(|bvid| VideoCard { bvid: (*bvid).into(), ..Default::default() }).collect()
    }

    #[tokio::test]
    async fn a_list_already_playing_keeps_its_order() {
        let shared = lineup();
        let heard = Listening::new(shared.clone(), None);
        shared.put(TEMP_PLAYLIST, "收藏夹", cards(&["a", "b", "c"]));
        shared.set_active(TEMP_PLAYLIST);
        assert_eq!(heard.start_here("b").await.unwrap().bvid, "b");
        assert_eq!(shared.active(), TEMP_PLAYLIST, "相关推荐 did not take the list over");
        assert_eq!(heard.items().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn a_video_off_the_list_falls_back_to_related() {
        let shared = lineup();
        let heard = Listening::new(shared.clone(), None);
        shared.put(TEMP_PLAYLIST, "收藏夹", cards(&["a"]));
        shared.set_active(TEMP_PLAYLIST);
        heard.start_here("z").await.unwrap();
        assert_eq!(shared.active(), RELATED_PLAYLIST);
    }

    #[tokio::test]
    async fn a_list_handed_over_is_the_one_heard() {
        let shared = lineup();
        let heard = Listening::new(shared.clone(), None);
        let bvids: Vec<String> = ["a", "b", "c"].iter().map(|bvid| (*bvid).to_owned()).collect();
        heard.start_listing("收藏夹", &bvids, "b").await.unwrap();
        assert_eq!(shared.active(), TEMP_PLAYLIST);
        assert_eq!(heard.current().unwrap().bvid, "b");
        assert_eq!(heard.items().unwrap().len(), 3);
        assert_eq!(heard.next(true).unwrap().unwrap().bvid, "c", "it follows the list, not 相关推荐");
    }

    #[tokio::test]
    async fn related_plays_in_the_mode_that_was_chosen() {
        let shared = lineup();
        let heard = Listening::new(shared.clone(), None);
        heard.start_related("a").await.unwrap();
        heard.append(&["b".into(), "c".into()]).await;
        heard.set_mode(PlaylistMode::LoopOne).unwrap();
        assert_eq!(heard.mode().unwrap(), PlaylistMode::LoopOne, "相关推荐 takes the mode too");
        assert_eq!(heard.next(false).unwrap().unwrap().bvid, "a", "单曲循环 keeps the one being heard");
        heard.set_mode(PlaylistMode::Shuffle).unwrap();
        assert_eq!(heard.mode().unwrap(), PlaylistMode::Shuffle);
    }
}
