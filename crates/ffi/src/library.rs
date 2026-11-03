//! History, watch later, favourites.

use crate::{FfiFavFolder, FfiFavPage, FfiHistoryItem, FfiHistoryPage, FfiVideoCard, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// Watch history; pass the returned cursor back for the next page.
    pub async fn history(&self, cursor: Option<String>) -> Res<FfiHistoryPage> {
        let (items, cursor) = self.library.history(cursor.as_deref()).await?;
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

    /// 稍后再看, with how far each was watched (`progress_s` -1: to the end) and when added.
    pub async fn watch_later_list(&self) -> Res<Vec<FfiHistoryItem>> {
        Ok(self
            .library
            .watch_later()
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
        Ok(self.library.remove_watch_later(&aids).await?)
    }

    /// Takes the videos watched to the end out of 稍后再看.
    pub async fn clear_watched_later(&self) -> Res<()> {
        Ok(self.library.clear_watched_later().await?)
    }

    /// The user's own favourite folders.
    /// A new 收藏夹; returns its id.
    pub async fn create_fav_folder(&self, title: String, intro: String, private: bool) -> Res<u64> {
        Ok(self.library.create_folder(&title, &intro, private).await?)
    }

    pub async fn edit_fav_folder(&self, id: u64, title: String, intro: String, private: bool) -> Res<()> {
        Ok(self.library.edit_folder(id, &title, &intro, private).await?)
    }

    /// Takes videos (by aid) out of a 收藏夹.
    pub async fn remove_from_fav_folder(&self, folder: u64, aids: Vec<u64>) -> Res<()> {
        Ok(self.library.remove_from_folder(folder, &aids).await?)
    }

    /// Moves videos to another 收藏夹, or copies them when `keep`.
    pub async fn transfer_fav(&self, from: u64, to: u64, aids: Vec<u64>, keep: bool) -> Res<()> {
        Ok(self.library.transfer(from, to, &aids, keep).await?)
    }

    pub async fn delete_fav_folder(&self, id: u64) -> Res<()> {
        Ok(self.library.delete_folder(id).await?)
    }

    /// A folder's 简介, for editing it.
    pub async fn fav_folder_intro(&self, id: u64) -> Res<String> {
        Ok(self.library.folder_intro(id).await?)
    }

    pub async fn my_fav_folders(&self) -> Res<Vec<FfiFavFolder>> {
        Ok(self.library.fav_folders().await?.into_iter().map(Into::into).collect())
    }

    /// Page `pn` (from 1) of a favourite folder.
    pub async fn fav_items(&self, folder_id: u64, pn: u32) -> Res<FfiFavPage> {
        let (cards, has_more) = self.library.fav_items(folder_id, pn).await?;
        Ok(FfiFavPage { items: cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(), has_more })
    }

    /// Reports playback so the video lands in 历史记录. `play_type`: 1 start, 0 playing,
    /// 2 pause / leaving. Guests are skipped silently.
    pub async fn report_watch(&self, bvid: String, cid: u64, played_s: u64, realtime_s: u64, play_type: u8) -> Res<()> {
        Ok(self.library.report_watch(&bvid, cid, played_s, realtime_s, play_type).await?)
    }
}
