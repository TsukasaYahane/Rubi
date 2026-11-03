//! The user's own lists on B站: history (and reporting to it), watch later, favourites.

use std::sync::Arc;

use rubi_api::{ActionApi, LibraryApi};
use rubi_model::{FavFolder, HistoryEntry, VideoCard};

use crate::error::Result;
use crate::ports::{LoginState, VideoLookup};
use crate::videos::CardCache;

pub struct Library {
    api: Arc<dyn LibraryApi>,
    folders: Arc<dyn ActionApi>,
    login: Arc<dyn LoginState>,
    videos: Arc<dyn VideoLookup>,
    cards: Arc<CardCache>,
}

impl Library {
    pub(crate) fn new(
        api: Arc<dyn LibraryApi>,
        folders: Arc<dyn ActionApi>,
        login: Arc<dyn LoginState>,
        videos: Arc<dyn VideoLookup>,
        cards: Arc<CardCache>,
    ) -> Self {
        Self { api, folders, login, videos, cards }
    }

    /// Watch history; pass the returned cursor back for the next page. Its cards are not
    /// cached: they carry the part last watched instead of the first, and no counts, so a
    /// video opened from here is looked up in full.
    pub async fn history(&self, cursor: Option<&str>) -> Result<(Vec<HistoryEntry>, Option<String>)> {
        Ok(self.api.history(cursor).await?)
    }

    /// 稍后再看 with how far each was watched.
    pub async fn watch_later(&self) -> Result<Vec<HistoryEntry>> {
        let items = self.api.watch_later_list().await?;
        self.cards.remember(&items.iter().map(|h| h.card.clone()).collect::<Vec<_>>());
        Ok(items)
    }

    /// Takes videos (by aid) out of 稍后再看, one request each, as B站 removes them.
    pub async fn remove_watch_later(&self, aids: &[u64]) -> Result<()> {
        for &aid in aids {
            self.folders.remove_watch_later(aid).await?;
        }
        Ok(())
    }

    /// Takes the videos watched to the end out of 稍后再看.
    pub async fn clear_watched_later(&self) -> Result<()> {
        Ok(self.folders.clear_watched_later().await?)
    }

    /// The user's own favourite folders.
    pub async fn fav_folders(&self) -> Result<Vec<FavFolder>> {
        Ok(self.folders.fav_folders(0).await?)
    }

    /// Page `pn` (from 1) of a favourite folder; also whether more follow.
    /// A new folder (title trimmed, 1–20 characters); returns its id.
    pub async fn create_folder(&self, title: &str, intro: &str, private: bool) -> Result<u64> {
        let title = folder_title(title)?;
        Ok(self.folders.create_folder(&title, intro.trim(), private).await?)
    }

    pub async fn edit_folder(&self, id: u64, title: &str, intro: &str, private: bool) -> Result<()> {
        let title = folder_title(title)?;
        Ok(self.folders.edit_folder(id, &title, intro.trim(), private).await?)
    }

    /// Takes videos (by aid) out of `folder`.
    pub async fn remove_from_folder(&self, folder: u64, aids: &[u64]) -> Result<()> {
        if aids.is_empty() {
            return Ok(());
        }
        Ok(self.folders.remove_from_folder(folder, aids).await?)
    }

    /// Moves videos to another folder, or copies them there when `keep`.
    pub async fn transfer(&self, from: u64, to: u64, aids: &[u64], keep: bool) -> Result<()> {
        if aids.is_empty() || from == to {
            return Ok(());
        }
        Ok(self.folders.transfer(from, to, aids, keep).await?)
    }

    pub async fn delete_folder(&self, id: u64) -> Result<()> {
        Ok(self.folders.delete_folder(id).await?)
    }

    pub async fn folder_intro(&self, id: u64) -> Result<String> {
        Ok(self.folders.folder_intro(id).await?)
    }

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
        if !self.login.is_logged_in() {
            return Ok(());
        }
        let card = self.videos.card(bvid).await?;
        Ok(self.api.heartbeat(card.aid, cid, played_s, realtime_s, play_type).await?)
    }
}

/// Longest folder title B站 takes.
const MAX_FOLDER_TITLE: usize = 20;

fn folder_title(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() {
        return Err(crate::error::CoreError::Invalid("请输入收藏夹名称".into()));
    }
    if title.chars().count() > MAX_FOLDER_TITLE {
        return Err(crate::error::CoreError::Invalid(format!("名称最多 {MAX_FOLDER_TITLE} 个字")));
    }
    Ok(title.to_owned())
}
