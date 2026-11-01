//! 离线缓存: videos copied to the device for playback without a network. Each download keeps
//! its video and audio streams as files under its own folder, resumes from what is on disk,
//! and is listed from the store across restarts.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rubi_api::{MediaFetch, PlayUrl, Stream};
use rubi_model::{Download, DownloadState, now_secs};
use rubi_store::{DownloadRepo, SqliteStore};
use tokio::task::AbortHandle;

use crate::error::{CoreError, Result};
use crate::videos::Videos;

const VIDEO_FILE: &str = "video.m4s";
const AUDIO_FILE: &str = "audio.m4s";

/// Where a finished download's streams are.
#[derive(Debug, Clone)]
pub struct LocalCopy {
    pub video: PathBuf,
    /// `None` for the rare stream that carries its own sound.
    pub audio: Option<PathBuf>,
}

/// The streams a copy is made of: the video stream of the quality granted (AVC preferred, as
/// the widest supported), and the best audio.
pub fn pick_streams(url: &PlayUrl) -> (Option<&Stream>, Option<&Stream>) {
    let granted: Vec<&Stream> = url.video.iter().filter(|s| s.id == url.quality).collect();
    let pool: Vec<&Stream> = if granted.is_empty() { url.video.iter().collect() } else { granted };
    let video = pool.iter().find(|s| s.codecs.starts_with("avc1")).or_else(|| pool.iter().max_by_key(|s| s.bandwidth));
    (video.copied(), url.audio.iter().max_by_key(|s| s.bandwidth))
}

pub struct Downloads {
    inner: Arc<Inner>,
}

struct Inner {
    videos: Arc<Videos>,
    media: Arc<dyn MediaFetch>,
    store: Arc<SqliteStore>,
    dir: PathBuf,
    tasks: Mutex<HashMap<String, AbortHandle>>,
    /// Bytes written so far by running downloads (the store is updated when one stops).
    live: Mutex<HashMap<String, u64>>,
}

impl Downloads {
    pub(crate) fn new(videos: Arc<Videos>, media: Arc<dyn MediaFetch>, store: Arc<SqliteStore>, dir: PathBuf) -> Self {
        Self { inner: Arc::new(Inner { videos, media, store, dir, tasks: Mutex::default(), live: Mutex::default() }) }
    }

    /// Every download, newest first, with live progress for the running ones.
    pub fn list(&self) -> Result<Vec<Download>> {
        let live = self.inner.live.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let mut all = self.inner.store.downloads()?;
        for d in &mut all {
            if let Some(&done) = live.get(&d.card.bvid) {
                d.done_bytes = done;
            }
        }
        Ok(all)
    }

    /// Starts (or resumes) copying `bvid` at quality `qn`. Must be called inside the runtime.
    pub async fn start(&self, bvid: &str, qn: u32) -> Result<()> {
        if self.is_running(bvid) {
            return Ok(());
        }
        let existing = self.inner.store.downloads()?.into_iter().find(|d| d.card.bvid == bvid);
        let record = match existing {
            Some(mut d) => {
                d.state = DownloadState::Queued;
                d
            }
            None => Download {
                card: self.inner.videos.card(bvid).await?,
                quality: qn,
                state: DownloadState::Queued,
                done_bytes: 0,
                total_bytes: 0,
                created_at: now_secs(),
            },
        };
        self.inner.store.put_download(&record)?;
        let inner = self.inner.clone();
        let id = bvid.to_owned();
        let task = tokio::spawn(async move {
            let outcome = inner.run(&id, record.quality).await;
            inner.finish(&id, outcome);
        });
        self.inner.tasks.lock().unwrap_or_else(|e| e.into_inner()).insert(bvid.to_owned(), task.abort_handle());
        Ok(())
    }

    /// Stops a running download; what is on disk stays for resuming.
    pub fn pause(&self, bvid: &str) -> Result<()> {
        self.stop_task(bvid);
        self.inner.set_state(bvid, DownloadState::Paused)
    }

    /// Stops it and deletes its files and record.
    pub fn remove(&self, bvid: &str) -> Result<()> {
        self.stop_task(bvid);
        let folder = self.inner.folder(bvid);
        if folder.exists() {
            fs::remove_dir_all(folder)?;
        }
        Ok(self.inner.store.remove_download(bvid)?)
    }

    /// The files of a finished download.
    pub fn local(&self, bvid: &str) -> Result<Option<LocalCopy>> {
        let done =
            self.inner.store.downloads()?.into_iter().any(|d| d.card.bvid == bvid && d.state == DownloadState::Done);
        if !done {
            return Ok(None);
        }
        let folder = self.inner.folder(bvid);
        let video = folder.join(VIDEO_FILE);
        let audio = folder.join(AUDIO_FILE);
        Ok(video.exists().then(|| LocalCopy { video, audio: audio.exists().then_some(audio) }))
    }

    fn is_running(&self, bvid: &str) -> bool {
        self.inner.tasks.lock().unwrap_or_else(|e| e.into_inner()).get(bvid).is_some_and(|t| !t.is_finished())
    }

    fn stop_task(&self, bvid: &str) {
        if let Some(t) = self.inner.tasks.lock().unwrap_or_else(|e| e.into_inner()).remove(bvid) {
            t.abort();
        }
        self.inner.live.lock().unwrap_or_else(|e| e.into_inner()).remove(bvid);
    }
}

impl Inner {
    fn folder(&self, bvid: &str) -> PathBuf {
        // BV ids are letters and digits only; anything else is refused before it reaches a path.
        self.dir.join(bvid.chars().filter(char::is_ascii_alphanumeric).collect::<String>())
    }

    fn set_state(&self, bvid: &str, state: DownloadState) -> Result<()> {
        let Some(mut d) = self.store.downloads()?.into_iter().find(|d| d.card.bvid == bvid) else { return Ok(()) };
        d.state = state;
        Ok(self.store.put_download(&d)?)
    }

    async fn run(&self, bvid: &str, qn: u32) -> Result<(u64, u64)> {
        self.set_state(bvid, DownloadState::Running)?;
        // Stream URLs expire, so every start asks for fresh ones.
        let playback = self.videos.playback(bvid, qn).await?;
        let (video, audio) = pick_streams(&playback.url);
        let video = video.ok_or_else(|| CoreError::Invalid("这个视频没有可缓存的画面".into()))?;
        let seconds = playback.url.duration_ms / 1000;
        let estimate = |s: &Stream| s.bandwidth / 8 * seconds;
        let folder = self.folder(bvid);
        fs::create_dir_all(&folder)?;

        let audio_estimate = audio.map(estimate).unwrap_or_default();
        let video_total =
            self.fetch(bvid, video, &folder.join(VIDEO_FILE), 0, estimate(video) + audio_estimate).await?;
        let audio_total = match audio {
            Some(a) => self.fetch(bvid, a, &folder.join(AUDIO_FILE), video_total, video_total + audio_estimate).await?,
            None => 0,
        };
        Ok((video_total + audio_total, video_total + audio_total))
    }

    /// Copies one stream into `path`, resuming after what is there; tries the backup urls in
    /// turn. `before` is what earlier streams of this download already hold. Returns the
    /// stream's size.
    async fn fetch(&self, bvid: &str, stream: &Stream, path: &Path, before: u64, total_hint: u64) -> Result<u64> {
        let mut last_error = None;
        for url in std::iter::once(&stream.url).chain(&stream.backup_urls) {
            match self.fetch_from(bvid, url, path, before, total_hint).await {
                Ok(size) => return Ok(size),
                Err(e) => last_error = Some(e),
            }
        }
        Err(last_error.unwrap_or_else(|| CoreError::Invalid("没有可用的地址".into())))
    }

    async fn fetch_from(&self, bvid: &str, url: &str, path: &Path, before: u64, total_hint: u64) -> Result<u64> {
        let have = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let mut body = self.media.open(url, have).await?;
        let (mut file, mut written) =
            if body.resumed() { (OpenOptions::new().append(true).open(path)?, have) } else { (File::create(path)?, 0) };
        let size = body.total().unwrap_or(total_hint.saturating_sub(before));
        self.update_total(bvid, before + size.max(written))?;
        while let Some(chunk) = body.chunk().await? {
            file.write_all(&chunk)?;
            written += chunk.len() as u64;
            self.live.lock().unwrap_or_else(|e| e.into_inner()).insert(bvid.to_owned(), before + written);
        }
        file.flush()?;
        Ok(written)
    }

    fn update_total(&self, bvid: &str, total: u64) -> Result<()> {
        let Some(mut d) = self.store.downloads()?.into_iter().find(|d| d.card.bvid == bvid) else { return Ok(()) };
        d.total_bytes = d.total_bytes.max(total);
        Ok(self.store.put_download(&d)?)
    }

    fn finish(&self, bvid: &str, outcome: Result<(u64, u64)>) {
        self.tasks.lock().unwrap_or_else(|e| e.into_inner()).remove(bvid);
        let done = self.live.lock().unwrap_or_else(|e| e.into_inner()).remove(bvid);
        let Ok(Some(mut d)) = self.store.downloads().map(|all| all.into_iter().find(|d| d.card.bvid == bvid)) else {
            return;
        };
        match outcome {
            Ok((done_bytes, total)) => {
                d.state = DownloadState::Done;
                d.done_bytes = done_bytes;
                d.total_bytes = total;
            }
            Err(e) => {
                d.state = DownloadState::Failed(e.to_string());
                if let Some(b) = done {
                    d.done_bytes = b;
                }
            }
        }
        let _ = self.store.put_download(&d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(id: u32, codecs: &str, bandwidth: u64) -> Stream {
        Stream {
            id,
            url: format!("u{id}{codecs}"),
            backup_urls: vec![],
            bandwidth,
            codecs: codecs.into(),
            width: 0,
            height: 0,
        }
    }

    #[test]
    fn picks_avc_of_the_granted_quality_and_the_best_audio() {
        let url = PlayUrl {
            duration_ms: 1000,
            quality: 80,
            accept: vec![],
            video: vec![stream(80, "hev1", 900), stream(80, "avc1.640032", 800), stream(64, "avc1", 400)],
            audio: vec![stream(30216, "mp4a", 64), stream(30280, "mp4a", 192)],
        };
        let (v, a) = pick_streams(&url);
        assert_eq!(v.unwrap().codecs, "avc1.640032");
        assert_eq!(a.unwrap().bandwidth, 192);
    }
}
