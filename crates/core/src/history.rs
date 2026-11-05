//! 历史记录: the user's watch history, and reporting playback into it.

use std::sync::Arc;

use rubi_api::HistoryApi;
use rubi_model::HistoryEntry;

use crate::error::Result;
use crate::ports::{LoginState, VideoLookup};

pub struct History {
    api: Arc<dyn HistoryApi>,
    login: Arc<dyn LoginState>,
    videos: Arc<dyn VideoLookup>,
}

impl History {
    pub(crate) fn new(api: Arc<dyn HistoryApi>, login: Arc<dyn LoginState>, videos: Arc<dyn VideoLookup>) -> Self {
        Self { api, login, videos }
    }

    /// Watch history; pass the returned cursor back for the next page. Its cards are not
    /// cached: they carry the part last watched instead of the first, and no counts, so a
    /// video opened from here is looked up in full.
    pub async fn page(&self, cursor: Option<&str>) -> Result<(Vec<HistoryEntry>, Option<String>)> {
        Ok(self.api.history(cursor).await?)
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
