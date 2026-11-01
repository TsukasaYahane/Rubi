//! 离线缓存.

use rubi_model::DownloadState;

use crate::{FfiDownload, FfiDownloadState, FfiLocalCopy, FfiVideoCard, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// Every download, newest first, with live progress.
    pub fn downloads(&self) -> Res<Vec<FfiDownload>> {
        Ok(self
            .downloads
            .list()?
            .into_iter()
            .map(|d| {
                let (state, error) = match d.state {
                    DownloadState::Queued => (FfiDownloadState::Queued, None),
                    DownloadState::Running => (FfiDownloadState::Running, None),
                    DownloadState::Paused => (FfiDownloadState::Paused, None),
                    DownloadState::Done => (FfiDownloadState::Done, None),
                    DownloadState::Failed(e) => (FfiDownloadState::Failed, Some(e)),
                };
                FfiDownload {
                    card: FfiVideoCard::of(&d.card, None),
                    quality: d.quality,
                    state,
                    error,
                    done_bytes: d.done_bytes,
                    total_bytes: d.total_bytes,
                    created_at: d.created_at,
                }
            })
            .collect())
    }

    /// Starts (or resumes) copying a video at quality `qn`.
    pub async fn download_start(&self, bvid: String, qn: u32) -> Res<()> {
        Ok(self.downloads.start(&bvid, qn).await?)
    }

    pub fn download_pause(&self, bvid: String) -> Res<()> {
        Ok(self.downloads.pause(&bvid)?)
    }

    /// Deletes a download and its files.
    pub fn download_remove(&self, bvid: String) -> Res<()> {
        Ok(self.downloads.remove(&bvid)?)
    }

    /// The files of a finished download, for playing offline.
    pub fn download_local(&self, bvid: String) -> Res<Option<FfiLocalCopy>> {
        Ok(self.downloads.local(&bvid)?.map(|c| FfiLocalCopy {
            video_path: c.video.to_string_lossy().into_owned(),
            audio_path: c.audio.map(|a| a.to_string_lossy().into_owned()),
        }))
    }
}
