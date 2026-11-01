//! The user's own lists on B站: history (and reporting to it), watch later, favourites.

use std::sync::Arc;

use rubi_api::{ActionApi, LibraryApi};
use rubi_model::{FavFolder, HistoryEntry, VideoCard};

use crate::account::Account;
use crate::error::Result;
use crate::videos::{CardCache, Videos};

pub struct Library {
    api: Arc<dyn LibraryApi>,
    folders: Arc<dyn ActionApi>,
    account: Arc<Account>,
    videos: Arc<Videos>,
    cards: Arc<CardCache>,
}

impl Library {
    pub(crate) fn new(
        api: Arc<dyn LibraryApi>,
        folders: Arc<dyn ActionApi>,
        account: Arc<Account>,
        videos: Arc<Videos>,
        cards: Arc<CardCache>,
    ) -> Self {
        Self { api, folders, account, videos, cards }
    }

    /// Watch history; pass the returned cursor back for the next page.
    pub async fn history(&self, cursor: Option<&str>) -> Result<(Vec<HistoryEntry>, Option<String>)> {
        let (items, cursor) = self.api.history(cursor).await?;
        self.cards.remember(&items.iter().map(|h| h.card.clone()).collect::<Vec<_>>());
        Ok((items, cursor))
    }

    pub async fn watch_later(&self) -> Result<Vec<VideoCard>> {
        let cards = self.api.watch_later_list().await?;
        self.cards.remember(&cards);
        Ok(cards)
    }

    /// The user's own favourite folders.
    pub async fn fav_folders(&self) -> Result<Vec<FavFolder>> {
        Ok(self.folders.fav_folders(0).await?)
    }

    /// Page `pn` (from 1) of a favourite folder; also whether more follow.
    pub async fn fav_items(&self, folder_id: u64, pn: u32) -> Result<(Vec<VideoCard>, bool)> {
        let (cards, more) = self.api.fav_items(folder_id, pn.max(1)).await?;
        self.cards.remember(&cards);
        Ok((cards, more))
    }

    /// Reports playback so the video lands in 历史记录. `play_type`: 1 start, 0 playing,
    /// 2 pause / leaving. Guests are skipped silently.
    pub async fn report_watch(
        &self,
        bvid: &str,
        cid: u64,
        played_s: u64,
        realtime_s: u64,
        play_type: u8,
    ) -> Result<()> {
        if !self.account.is_logged_in() {
            return Ok(());
        }
        let card = self.videos.card(bvid).await?;
        Ok(self.api.heartbeat(card.aid, cid, played_s, realtime_s, play_type).await?)
    }
}
