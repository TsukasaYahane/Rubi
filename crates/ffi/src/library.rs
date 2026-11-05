//! History, watch later, favourites.

use crate::{
    FavoritesService, FfiFavFolder, FfiFavPage, FfiHistoryItem, FfiHistoryPage, FfiVideoCard, HistoryService, Res,
    WatchLaterService,
};

#[uniffi::export(async_runtime = "tokio")]
impl HistoryService {
    /// Watch history; pass the returned cursor back for the next page.
    pub async fn history(&self, cursor: Option<String>) -> Res<FfiHistoryPage> {
        let (items, cursor) = self.history.page(cursor.as_deref()).await?;
        let items = items
            .iter()
            .map(|h| FfiHistoryItem {
                card: FfiVideoCard::of(&h.card, None),
                progress_s: h.progress_s,
                viewed_at: h.viewed_at,
            })
            .collect();
        Ok(FfiHistoryPage { items, cursor })
    }

    /// Reports playback so the video lands in 历史记录. `play_type`: 1 start, 0 playing,
    /// 2 pause / leaving. Guests are skipped silently.
    pub async fn report_watch(&self, bvid: String, cid: u64, played_s: u64, realtime_s: u64, play_type: u8) -> Res<()> {
        Ok(self.history.report_watch(&bvid, cid, played_s, realtime_s, play_type).await?)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl WatchLaterService {
    /// 稍后再看, with how far each was watched (`progress_s` -1: to the end) and when added.
    pub async fn watch_later_list(&self) -> Res<Vec<FfiHistoryItem>> {
        Ok(self
            .watch_later
            .list()
            .await?
            .iter()
            .map(|h| FfiHistoryItem {
                card: FfiVideoCard::of(&h.card, None),
                progress_s: h.progress_s,
                viewed_at: h.viewed_at,
            })
            .collect())
    }

    /// Takes videos (by aid) out of 稍后再看.
    pub async fn remove_watch_later(&self, aids: Vec<u64>) -> Res<()> {
        Ok(self.watch_later.remove(&aids).await?)
    }

    /// Takes the videos watched to the end out of 稍后再看.
    pub async fn clear_watched_later(&self) -> Res<()> {
        Ok(self.watch_later.clear_watched().await?)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl FavoritesService {
    /// The user's own favourite folders.
    /// A new 收藏夹; returns its id.
    pub async fn create_fav_folder(&self, title: String, intro: String, private: bool) -> Res<u64> {
        Ok(self.favorites.create_folder(&title, &intro, private).await?)
    }

    pub async fn edit_fav_folder(&self, id: u64, title: String, intro: String, private: bool) -> Res<()> {
        Ok(self.favorites.edit_folder(id, &title, &intro, private).await?)
    }

    /// Takes videos (by aid) out of a 收藏夹.
    pub async fn remove_from_fav_folder(&self, folder: u64, aids: Vec<u64>) -> Res<()> {
        Ok(self.favorites.remove_from_folder(folder, &aids).await?)
    }

    /// Moves videos to another 收藏夹, or copies them when `keep`.
    pub async fn transfer_fav(&self, from: u64, to: u64, aids: Vec<u64>, keep: bool) -> Res<()> {
        Ok(self.favorites.transfer(from, to, &aids, keep).await?)
    }

    pub async fn delete_fav_folder(&self, id: u64) -> Res<()> {
        Ok(self.favorites.delete_folder(id).await?)
    }

    /// A folder's 简介, for editing it.
    pub async fn fav_folder_intro(&self, id: u64) -> Res<String> {
        Ok(self.favorites.folder_intro(id).await?)
    }

    pub async fn my_fav_folders(&self) -> Res<Vec<FfiFavFolder>> {
        Ok(self.favorites.folders().await?.into_iter().map(Into::into).collect())
    }

    /// Page `pn` (from 1) of a favourite folder.
    pub async fn fav_items(&self, folder_id: u64, pn: u32) -> Res<FfiFavPage> {
        let (cards, has_more) = self.favorites.items(folder_id, pn).await?;
        Ok(FfiFavPage { items: cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(), has_more })
    }
}
