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

    pub async fn watch_later_list(&self) -> Res<Vec<FfiVideoCard>> {
        Ok(self.library.watch_later().await?.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    /// The user's own favourite folders.
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
