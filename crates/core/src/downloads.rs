//! 离线缓存: videos copied to the device for playback without a network. Each download keeps
//! its video and audio streams as files under its own folder, resumes from what is on disk,
//! and is listed from the store across restarts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use rubi_api::{MediaFetch, PlayUrl, Stream};
use rubi_model::{Download, DownloadState, now_secs};
use rubi_store::DownloadRepo;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;

use crate::error::{CoreError, Result};
use crate::ports::VideoStreams;

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

/// A download's task: reserved (`handle: None`) from the moment `start` claims it, so a second
/// `start` of the same video cannot slip in while the first is still looking it up.
struct Task {
    id: u64,
    handle: Option<JoinHandle<()>>,
}

pub struct Downloads {
    inner: Arc<Inner>,
}

struct Inner {
    videos: Arc<dyn VideoStreams>,
    media: Arc<dyn MediaFetch>,
    store: Arc<dyn DownloadRepo + Send + Sync>,
    dir: PathBuf,
    tasks: Mutex<HashMap<String, Task>>,
    next_id: AtomicU64,
    /// Bytes written so far by running downloads (the store is updated when one stops).
    live: Mutex<HashMap<String, u64>>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Downloads {
    pub(crate) fn new(
        videos: Arc<dyn VideoStreams>,
        media: Arc<dyn MediaFetch>,
        store: Arc<dyn DownloadRepo + Send + Sync>,
        dir: PathBuf,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                videos,
                media,
                store,
                dir,
                tasks: Mutex::default(),
                next_id: AtomicU64::new(1),
                live: Mutex::default(),
            }),
        }
    }

    /// Every download, newest first, with live progress for the running ones.
    pub fn list(&self) -> Result<Vec<Download>> {
        let live = lock(&self.inner.live).clone();
        let mut all = self.inner.store.downloads()?;
        for d in &mut all {
            if let Some(&done) = live.get(&d.card.bvid) {
                d.done_bytes = done;
            }
        }
        Ok(all)
    }

    /// Starts (or resumes) copying `bvid` at quality `qn`. Must be called inside the runtime.
    /// Starting one that is already running (or starting) does nothing.
    pub async fn start(&self, bvid: &str, qn: u32) -> Result<()> {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        {
            let mut tasks = lock(&self.inner.tasks);
            if tasks.get(bvid).is_some_and(|t| t.handle.as_ref().is_none_or(|h| !h.is_finished())) {
                return Ok(());
            }
            tasks.insert(bvid.to_owned(), Task { id, handle: None });
        }
        let record = match self.prepare(bvid, qn).await {
            Ok(r) => r,
            Err(e) => {
                self.inner.release(bvid, id);
                return Err(e);
            }
        };
        let inner = self.inner.clone();
        let key = bvid.to_owned();
        let handle = tokio::spawn(async move {
            let outcome = inner.run(&key, record.quality).await;
            inner.finish(&key, id, outcome);
        });
        if let Some(t) = lock(&self.inner.tasks).get_mut(bvid).filter(|t| t.id == id) {
            t.handle = Some(handle);
        }
        Ok(())
    }

    /// The record to run: the stored one (queued again), else a new one.
    async fn prepare(&self, bvid: &str, qn: u32) -> Result<Download> {
        let mut queued = false;
        self.inner.store.update_download(bvid, &mut |d| {
            d.state = DownloadState::Queued;
            queued = true;
        })?;
        if queued {
            return self.inner.store.download(bvid)?.ok_or_else(|| CoreError::Invalid("下载记录已删除".into()));
        }
        let record = Download {
            card: self.inner.videos.card(bvid).await?,
            quality: qn,
            state: DownloadState::Queued,
            done_bytes: 0,
            total_bytes: 0,
            created_at: now_secs(),
        };
        self.inner.store.put_download(&record)?;
        Ok(record)
    }

    /// Stops a running download and waits for it to let go of its files; what is on disk
    /// stays for resuming.
    pub async fn pause(&self, bvid: &str) -> Result<()> {
        self.stop_task(bvid).await;
        self.inner.set_state(bvid, DownloadState::Paused)
    }

    /// Stops it, waits for it to stop writing, and deletes its files and record.
    pub async fn remove(&self, bvid: &str) -> Result<()> {
        self.stop_task(bvid).await;
        let folder = self.inner.folder(bvid);
        match tokio::fs::remove_dir_all(&folder).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        Ok(self.inner.store.remove_download(bvid)?)
    }

    /// The files of a finished download.
    pub fn local(&self, bvid: &str) -> Result<Option<LocalCopy>> {
        let done = self.inner.store.download(bvid)?.is_some_and(|d| d.state == DownloadState::Done);
        if !done {
            return Ok(None);
        }
        let folder = self.inner.folder(bvid);
        let video = folder.join(VIDEO_FILE);
        let audio = folder.join(AUDIO_FILE);
        Ok(video.exists().then(|| LocalCopy { video, audio: audio.exists().then_some(audio) }))
    }

    async fn stop_task(&self, bvid: &str) {
        let task = lock(&self.inner.tasks).remove(bvid);
        if let Some(handle) = task.and_then(|t| t.handle) {
            handle.abort();
            // An aborted task ends at its next await; wait for that so no write follows.
            let _ = handle.await;
        }
        lock(&self.inner.live).remove(bvid);
    }
}

impl Inner {
    fn folder(&self, bvid: &str) -> PathBuf {
        // BV ids are letters and digits only; anything else is refused before it reaches a path.
        self.dir.join(bvid.chars().filter(char::is_ascii_alphanumeric).collect::<String>())
    }

    /// Forgets the task reserved as `id`, unless a newer one took its place.
    fn release(&self, bvid: &str, id: u64) {
        let mut tasks = lock(&self.tasks);
        if tasks.get(bvid).is_some_and(|t| t.id == id) {
            tasks.remove(bvid);
        }
    }

    fn set_state(&self, bvid: &str, state: DownloadState) -> Result<()> {
        self.store.update_download(bvid, &mut |d| d.state = state.clone())?;
        Ok(())
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
        tokio::fs::create_dir_all(&folder).await?;

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
        let have = tokio::fs::metadata(path).await.map(|m| m.len()).unwrap_or(0);
        let mut body = self.media.open(url, have).await?;
        let (mut file, mut written) = if body.resumed() {
            (tokio::fs::OpenOptions::new().append(true).open(path).await?, have)
        } else {
            (tokio::fs::File::create(path).await?, 0)
        };
        let size = body.total().unwrap_or(total_hint.saturating_sub(before));
        self.update_total(bvid, before + size.max(written))?;
        lock(&self.live).insert(bvid.to_owned(), before + written);
        while let Some(chunk) = body.chunk().await? {
            file.write_all(&chunk).await?;
            written += chunk.len() as u64;
            if let Some(done) = lock(&self.live).get_mut(bvid) {
                *done = before + written;
            }
        }
        file.flush().await?;
        Ok(written)
    }

    fn update_total(&self, bvid: &str, total: u64) -> Result<()> {
        self.store.update_download(bvid, &mut |d| d.total_bytes = d.total_bytes.max(total))?;
        Ok(())
    }

    fn finish(&self, bvid: &str, id: u64, outcome: Result<(u64, u64)>) {
        self.release(bvid, id);
        let done = lock(&self.live).remove(bvid);
        let _ = self.store.update_download(bvid, &mut |d| match &outcome {
            Ok((done_bytes, total)) => {
                d.state = DownloadState::Done;
                d.done_bytes = *done_bytes;
                d.total_bytes = *total;
            }
            Err(e) => {
                d.state = DownloadState::Failed(e.to_string());
                if let Some(b) = done {
                    d.done_bytes = b;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rubi_api::MediaBody;
    use rubi_model::VideoCard;
    use rubi_store::SqliteStore;

    use crate::ports::VideoLookup;
    use crate::videos::Playback;

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

    struct Fake;

    #[async_trait]
    impl VideoLookup for Fake {
        async fn card(&self, bvid: &str) -> Result<VideoCard> {
            // Slow enough that a second start arrives while the first is still looking up.
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            Ok(VideoCard { bvid: bvid.into(), ..Default::default() })
        }
    }

    #[async_trait]
    impl VideoStreams for Fake {
        async fn playback(&self, _: &str, _: u32) -> Result<Playback> {
            Ok(Playback {
                cid: 1,
                url: PlayUrl {
                    duration_ms: 1000,
                    quality: 80,
                    accept: vec![],
                    video: vec![stream(80, "avc1", 8)],
                    audio: vec![],
                },
                headers: vec![],
            })
        }
    }

    struct Body(Vec<Vec<u8>>);

    #[async_trait]
    impl MediaBody for Body {
        fn total(&self) -> Option<u64> {
            Some(self.0.iter().map(|c| c.len() as u64).sum())
        }
        fn resumed(&self) -> bool {
            false
        }
        async fn chunk(&mut self) -> rubi_api::Result<Option<Vec<u8>>> {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            Ok((!self.0.is_empty()).then(|| self.0.remove(0)))
        }
    }

    #[async_trait]
    impl MediaFetch for Fake {
        async fn open(&self, _: &str, _: u64) -> rubi_api::Result<Box<dyn MediaBody>> {
            Ok(Box::new(Body(vec![b"abc".to_vec(), b"def".to_vec()])))
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn starts_once_finishes_and_removes() {
        let dir = std::env::temp_dir().join(format!("rubi-dl-test-{}", std::process::id()));
        let d =
            Downloads::new(Arc::new(Fake), Arc::new(Fake), Arc::new(SqliteStore::in_memory().unwrap()), dir.clone());
        let (a, b) = tokio::join!(d.start("BV1", 80), d.start("BV1", 80));
        a.unwrap();
        b.unwrap();
        for _ in 0..200 {
            if d.local("BV1").unwrap().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let copy = d.local("BV1").unwrap().expect("finished");
        assert_eq!(std::fs::read(&copy.video).unwrap(), b"abcdef", "written once, not twice");
        d.remove("BV1").await.unwrap();
        assert!(!copy.video.exists());
        assert!(d.list().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}
