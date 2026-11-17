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

/// What a finished copy is also written out as, outside Rubi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// Kept inside Rubi: the copy already is what 离线缓存 plays.
    #[default]
    Rubi,
    /// Picture and sound as one MP4.
    Mp4,
    /// The sound alone, as M4A.
    Audio,
}

impl ExportFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rubi => "仅 Rubi",
            Self::Mp4 => "MP4 视频",
            Self::Audio => "音频 M4A",
        }
    }
}

/// Which of a video's streams a copy takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Streams {
    /// Picture and sound.
    #[default]
    Both,
    /// The sound alone: half the bytes, and all an audio export needs.
    AudioOnly,
}

/// Where a finished download's streams are.
#[derive(Debug, Clone)]
pub struct LocalCopy {
    /// `None` for a copy that took the sound alone.
    pub video: Option<PathBuf>,
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
    /// A folder outside the app that finished copies were moved to, and are looked for in:
    /// a packaged app cannot write there itself, so the files are carried over.
    mirror: Mutex<Option<PathBuf>>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Copies `from` to `to`, saying which path could not be written when one cannot: an earlier
/// attempt's file is in the way otherwise, and Windows answers that with a bare number.
async fn copy_out(from: &Path, to: &Path) -> Result<()> {
    let _ = tokio::fs::remove_file(to).await;
    tokio::fs::copy(from, to).await.map_err(|e| CoreError::Invalid(format!("写不进 {}：{e}", to.display())))?;
    Ok(())
}

/// A title as a file name: what Windows refuses becomes underscores, and a long one is cut
/// short, so the write cannot fail on a character the title happened to hold.
fn file_stem(title: &str) -> String {
    let refused = r#"\/:*?"<>|"#;
    let cleaned: String = title.chars().map(|c| if refused.contains(c) || c.is_control() { '_' } else { c }).collect();
    let name: String = cleaned.trim().trim_matches('.').chars().take(80).collect();
    if name.is_empty() { "rubi".to_owned() } else { name }
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
                mirror: Mutex::default(),
            }),
        }
    }

    /// Picks up a folder that was moved in from elsewhere: every `BV…` folder that holds a file
    /// and has no record is written down as finished, so it shows in 离线缓存 again. The title is
    /// the bvid, because nothing on disk knows what the video was called.
    pub fn adopt(&self, dir: &Path) -> Result<usize> {
        let entries = std::fs::read_dir(dir)?;
        let mut adopted = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(bvid) = path.file_name().and_then(|name| name.to_str()).map(str::to_owned) else {
                continue;
            };
            if !bvid.starts_with("BV") || self.inner.store.download(&bvid)?.is_some() {
                continue;
            }
            let size = |name: &str| std::fs::metadata(path.join(name)).map(|m| m.len()).unwrap_or(0);
            let total = size(VIDEO_FILE) + size(AUDIO_FILE);
            if total == 0 {
                continue;
            }
            let record = Download {
                card: rubi_model::VideoCard { bvid: bvid.clone(), title: bvid.clone(), ..Default::default() },
                quality: 80,
                state: DownloadState::Done,
                done_bytes: total,
                total_bytes: total,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            };
            self.inner.store.put_download(&record)?;
            adopted += 1;
        }
        Ok(adopted)
    }

    /// Moves what is on disk to a new root, so 缓存位置 can change without leaving the files
    /// behind. The app's own copy stays where it is; this carries the user-facing one.
    pub fn relocate(&self, from: &Path, to: &Path) -> Result<usize> {
        if from == to {
            return Ok(0);
        }
        let entries = std::fs::read_dir(from)?;
        let mut moved = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name() else { continue };
            let target = to.join(name);
            if std::fs::create_dir_all(&target).is_err() {
                continue;
            }
            let mut carried = true;
            for file in [VIDEO_FILE, AUDIO_FILE] {
                let source = path.join(file);
                if !source.is_file() {
                    continue;
                }
                if std::fs::copy(&source, target.join(file)).is_err() {
                    carried = false;
                }
            }
            // The old folder goes only once every file reached the new one: half a moved video is
            // worse than a copy left behind.
            if carried {
                let _ = std::fs::remove_dir_all(&path);
            }
            moved += 1;
        }
        Ok(moved)
    }

    /// Names a folder outside the app that finished copies are moved to, and looked for in.
    /// A packaged app cannot write there while it downloads, so the files are carried over and
    /// the app's own copy is dropped; this is where the rest of the app then finds them.
    pub fn set_mirror(&self, root: Option<PathBuf>) {
        *lock(&self.inner.mirror) = root;
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

    /// Starts (or resumes) copying `bvid` at quality `qn`, taking `streams` of it. Must be
    /// called inside the runtime. Starting one that is already running (or starting) does
    /// nothing.
    pub async fn start(&self, bvid: &str, qn: u32, streams: Streams) -> Result<()> {
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
            let outcome = inner.run(&key, record.quality, streams).await;
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
        // The bytes written so far go into the record before the task lets go of them, or the
        // bar would fall back to wherever the last run ended.
        let written = lock(&self.inner.live).get(bvid).copied();
        self.stop_task(bvid).await;
        self.inner.store.update_download(bvid, &mut |d| {
            d.state = DownloadState::Paused;
            if let Some(written) = written {
                d.done_bytes = written;
            }
        })?;
        Ok(())
    }

    /// Stops it, waits for it to stop writing, and deletes its files and record.
    pub async fn remove(&self, bvid: &str) -> Result<()> {
        self.stop_task(bvid).await;
        let here = self.inner.folder(bvid);
        // A finished copy may have been carried off already; it goes with the record, or the
        // files would stay behind with nothing left to point at them.
        let moved = lock(&self.inner.mirror).as_ref().and_then(|root| here.file_name().map(|name| root.join(name)));
        for folder in std::iter::once(here).chain(moved) {
            match tokio::fs::remove_dir_all(&folder).await {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        Ok(self.inner.store.remove_download(bvid)?)
    }

    /// The files of a finished download, here or wherever they were moved to.
    pub fn local(&self, bvid: &str) -> Result<Option<LocalCopy>> {
        let done = self.inner.store.download(bvid)?.is_some_and(|d| d.state == DownloadState::Done);
        if !done {
            return Ok(None);
        }
        Ok(self.files(bvid))
    }

    /// The files of a copy wherever they are, finished or not: a run that stopped one answer
    /// short still leaves the streams on disk whole.
    pub fn files(&self, bvid: &str) -> Option<LocalCopy> {
        let here = self.inner.folder(bvid);
        if let Some(copy) = self.inner.copy_in(&here) {
            return Some(copy);
        }
        lock(&self.inner.mirror)
            .as_ref()
            .and_then(|root| here.file_name().and_then(|name| self.inner.copy_in(&root.join(name))))
    }

    /// The files still in the app's own folder, before anything has carried them off.
    pub fn local_here(&self, bvid: &str) -> Result<Option<LocalCopy>> {
        Ok(self.inner.copy_in(&self.inner.folder(bvid)))
    }

    /// Writes a finished copy out into `dir` as `format`, and says what it wrote. The streams
    /// move as they are: nothing is decoded or re-encoded here.
    pub async fn export(&self, bvid: &str, dir: &Path, format: ExportFormat) -> Result<Vec<PathBuf>> {
        if format == ExportFormat::Rubi {
            return Ok(Vec::new());
        }
        let Some(copy) = self.files(bvid) else {
            return Err(CoreError::Invalid("这一条还没有缓存好".into()));
        };
        let Some(source) = copy.video.as_ref().or(copy.audio.as_ref()).and_then(|p| p.parent()).map(Path::to_path_buf)
        else {
            return Err(CoreError::Invalid("这一条还没有缓存好".into()));
        };
        self.export_from(bvid, &source, dir, format).await
    }

    /// The same, for a copy whose streams are in `source` — a folder the caller found, rather
    /// than one the core was told about.
    pub async fn export_from(
        &self,
        bvid: &str,
        source: &Path,
        dir: &Path,
        format: ExportFormat,
    ) -> Result<Vec<PathBuf>> {
        if format == ExportFormat::Rubi {
            return Ok(Vec::new());
        }
        let Some(record) = self.inner.store.download(bvid)? else {
            return Err(CoreError::Invalid("下载记录已删除".into()));
        };
        let (picture, sound) = (source.join(VIDEO_FILE), source.join(AUDIO_FILE));
        let video = picture.exists().then_some(picture);
        let audio = sound.exists().then_some(sound);
        let stem = file_stem(&record.card.title);
        tokio::fs::create_dir_all(dir).await?;
        let written = match format {
            ExportFormat::Rubi => Vec::new(),
            ExportFormat::Audio => {
                let Some(from) = audio.as_deref() else {
                    return Err(CoreError::Invalid("这一条没有音轨".into()));
                };
                let to = dir.join(format!("{stem}.m4a"));
                // The cached stream is fragmented, which ordinary players do not follow; the
                // samples are gathered into one `moov` here, without decoding them.
                let (from, into) = (from.to_path_buf(), to.clone());
                tokio::task::spawn_blocking(move || crate::remux::m4a(&from, &into))
                    .await
                    .map_err(|e| CoreError::Invalid(e.to_string()))?
                    .map_err(|e| CoreError::Invalid(format!("转成 m4a 失败：{e}")))?;
                vec![to]
            }
            ExportFormat::Mp4 => {
                let Some(picture) = video else {
                    return Err(CoreError::Invalid("这一条只缓存了音频".into()));
                };
                // The streams are joined in one MP4 once the remux lands; until then each goes
                // out on its own, named for what it holds.
                let to = dir.join(format!("{stem}.mp4"));
                copy_out(&picture, &to).await?;
                let mut written = vec![to];
                if let Some(from) = audio {
                    let sound = dir.join(format!("{stem}.m4a"));
                    copy_out(&from, &sound).await?;
                    written.push(sound);
                }
                written
            }
        };
        Ok(written)
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

    /// The two files in `folder`, if either is there.
    fn copy_in(&self, folder: &Path) -> Option<LocalCopy> {
        let (picture, sound) = (folder.join(VIDEO_FILE), folder.join(AUDIO_FILE));
        let video = picture.exists().then_some(picture);
        let audio = sound.exists().then_some(sound);
        (video.is_some() || audio.is_some()).then_some(LocalCopy { video, audio })
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

    async fn run(&self, bvid: &str, qn: u32, streams: Streams) -> Result<(u64, u64)> {
        self.set_state(bvid, DownloadState::Running)?;
        // 412 means B站 refused the request: usually a stream address it no longer honours, so
        // the whole attempt starts again with fresh ones rather than retrying the same urls.
        match self.run_once(bvid, qn, streams).await {
            Ok(done) => Ok(done),
            Err(first) if first.to_string().contains("412") => {
                tokio::time::sleep(std::time::Duration::from_millis(1_500)).await;
                self.run_once(bvid, qn, streams).await
            }
            Err(first) => Err(first),
        }
    }

    async fn run_once(&self, bvid: &str, qn: u32, streams: Streams) -> Result<(u64, u64)> {
        // Stream URLs expire, so every start asks for fresh ones.
        let playback = self.videos.playback(bvid, qn).await?;
        let (video, audio) = pick_streams(&playback.url);
        let video = match streams {
            Streams::Both => Some(video.ok_or_else(|| CoreError::Invalid("这个视频没有可缓存的画面".into()))?),
            Streams::AudioOnly => None,
        };
        // A copy that has only a picture would be silent: the sound is always taken.
        let audio = audio.ok_or_else(|| CoreError::Invalid("这个视频没有可缓存的音轨".into()))?;
        let seconds = playback.url.duration_ms / 1000;
        let estimate = |s: &Stream| s.bandwidth / 8 * seconds;
        let folder = self.folder(bvid);
        tokio::fs::create_dir_all(&folder).await?;
        let mut total = 0;
        if let Some(video) = video {
            total += self.fetch(bvid, video, &folder.join(VIDEO_FILE), 0, estimate(video) + estimate(audio)).await?;
        }
        let hint = total + estimate(audio);
        total += self.fetch(bvid, audio, &folder.join(AUDIO_FILE), total, hint).await?;
        Ok((total, total))
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
        let mut body = match self.media.open(url, have).await {
            Ok(body) => body,
            // 416: what is on disk is already all the server has. That is a finished copy, not a
            // failure — a download asked for twice ends here.
            Err(e) if have > 0 && e.to_string().contains("416") => {
                self.update_total(bvid, before + have)?;
                return Ok(have);
            }
            Err(e) => return Err(e.into()),
        };
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
        // A stream that stops early is not a finished one: the file would be short, and every
        // reader after this would take it for whole.
        if written < size {
            return Err(CoreError::Invalid(format!("下载不完整：{written} / {size}")));
        }
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
            segment: None,
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
                    audio: vec![stream(30280, "mp4a", 8)],
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

    #[test]
    fn titles_become_file_names() {
        assert_eq!(file_stem(r#"a/b:c*d?e"f<g>h|i"#), "a_b_c_d_e_f_g_h_i");
        assert_eq!(file_stem("  ..  "), "rubi");
        assert_eq!(file_stem(&"长".repeat(120)).chars().count(), 80);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn starts_once_finishes_and_removes() {
        let dir = std::env::temp_dir().join(format!("rubi-dl-test-{}", std::process::id()));
        let d =
            Downloads::new(Arc::new(Fake), Arc::new(Fake), Arc::new(SqliteStore::in_memory().unwrap()), dir.clone());
        let (a, b) = tokio::join!(d.start("BV1", 80, Streams::Both), d.start("BV1", 80, Streams::Both));
        a.unwrap();
        b.unwrap();
        for _ in 0..200 {
            if d.local("BV1").unwrap().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let copy = d.local("BV1").unwrap().expect("finished");
        let picture = copy.video.clone().expect("a picture");
        assert_eq!(std::fs::read(&picture).unwrap(), b"abcdef", "written once, not twice");
        // Out it goes, as it is: no re-encoding, so the bytes are the ones on disk.
        let out = dir.join("out");
        let written = d.export("BV1", &out, ExportFormat::Mp4).await.unwrap();
        assert_eq!(written.len(), 2, "the picture and the sound, each on its own for now");
        assert_eq!(std::fs::read(&written[0]).unwrap(), b"abcdef");
        assert!(written[0].to_string_lossy().ends_with(".mp4"));
        d.remove("BV1").await.unwrap();
        assert!(!picture.exists());
        assert!(d.list().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn audio_only_takes_the_sound_alone() {
        let dir = std::env::temp_dir().join(format!("rubi-dl-audio-{}", std::process::id()));
        let d =
            Downloads::new(Arc::new(Fake), Arc::new(Fake), Arc::new(SqliteStore::in_memory().unwrap()), dir.clone());
        d.start("BV2", 80, Streams::AudioOnly).await.unwrap();
        for _ in 0..200 {
            if d.local("BV2").unwrap().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let copy = d.local("BV2").unwrap().expect("finished");
        assert!(copy.video.is_none(), "the picture was never asked for");
        assert!(copy.audio.is_some());
        d.remove("BV2").await.unwrap();
        let _ = std::fs::remove_dir_all(dir);
    }
}
