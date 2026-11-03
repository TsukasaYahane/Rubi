use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use futures_util::future::join_all;
use rubi_api::{DanmakuApi, SEGMENT_MS, SendDanmaku};
use rubi_filter::danmaku::layout::{LayoutConfig, MeasuredWidths, Placement, layout};
use rubi_filter::danmaku::{DanmakuOptions, DanmakuOutput, process};
use rubi_model::{CommandDanmaku, Danmaku};

use crate::error::{CoreError, Result};
use crate::ports::RuleSource;

/// A video's danmaku, filtered by the rule table, and sending new ones.
pub struct Danmakus {
    api: Arc<dyn DanmakuApi>,
    rules: Arc<dyn RuleSource>,
}

impl Danmakus {
    pub(crate) fn new(api: Arc<dyn DanmakuApi>, rules: Arc<dyn RuleSource>) -> Self {
        Self { api, rules }
    }

    /// Everything at once. Prefer [`session`](Self::session) while playing.
    pub async fn all(&self, cid: u64, duration_ms: u32) -> Result<DanmakuOutput> {
        load_danmaku(self.api.clone(), cid, duration_ms, &self.options()?).await
    }

    /// A segment-by-segment loader: the part being watched first, then the rest.
    pub fn session(&self, cid: u64, duration_ms: u32) -> Result<DanmakuSession> {
        Ok(DanmakuSession::open(self.api.clone(), cid, duration_ms, self.options()?))
    }

    /// Posts a danmaku (text trimmed, empty refused) and returns it as it will show.
    pub async fn send(&self, mut req: SendDanmaku) -> Result<Danmaku> {
        req.text = req.text.trim().to_owned();
        if req.text.is_empty() {
            return Err(CoreError::Invalid("弹幕不能为空".into()));
        }
        let id = self.api.send(&req).await?;
        Ok(Danmaku {
            id,
            progress_ms: req.progress_ms,
            mode: req.mode,
            font_size: 25,
            color: req.color,
            content: req.text,
            ..Default::default()
        })
    }

    /// Track layout for a batch of danmaku. `widths[i]` is the rendered width of `dms[i]` (same
    /// unit as `viewport_w`; missing ones are estimated). At least one track, and a scroll of at
    /// least a second.
    pub fn layout(
        &self,
        dms: &[Danmaku],
        widths: Vec<f32>,
        viewport_w: f32,
        font_size: f32,
        tracks: u32,
        scroll_ms: u32,
    ) -> Vec<Placement> {
        let cfg = LayoutConfig {
            viewport_w,
            font_size,
            tracks: tracks.max(1) as usize,
            scroll_ms: scroll_ms.max(1000),
            ..LayoutConfig::default()
        };
        let measured = MeasuredWidths(dms.iter().map(|d| d.content.as_str()).zip(widths).collect());
        layout(dms, &cfg, &measured)
    }

    fn options(&self) -> Result<DanmakuOptions> {
        Ok((*self.rules.danmaku_options()).clone())
    }
}

/// Loads every danmaku segment of `cid` plus its command danmaku and runs the filters.
async fn load_danmaku(
    api: Arc<dyn DanmakuApi>,
    cid: u64,
    duration_ms: u32,
    opts: &DanmakuOptions,
) -> Result<DanmakuOutput> {
    let session = DanmakuSession::open(api, cid, duration_ms, opts.clone());
    let all: Vec<u32> = (1..=session.segment_count()).collect();
    session.load(&all).await
}

#[derive(Default)]
struct Loaded {
    segments: BTreeMap<u32, Vec<Danmaku>>,
    commands: Option<Vec<CommandDanmaku>>,
}

/// Loads a video's danmaku segment by segment, so the part being watched can be shown
/// before the rest arrives. Every [`load`](Self::load) returns the filtered result over
/// everything loaded so far (jump merging needs the whole set to stay consistent).
pub struct DanmakuSession {
    api: Arc<dyn DanmakuApi>,
    cid: u64,
    duration_ms: u32,
    opts: DanmakuOptions,
    loaded: Mutex<Loaded>,
}

impl DanmakuSession {
    pub fn open(api: Arc<dyn DanmakuApi>, cid: u64, duration_ms: u32, opts: DanmakuOptions) -> Self {
        Self { api, cid, duration_ms, opts, loaded: Mutex::default() }
    }

    /// Six-minute segments covering the video (at least one).
    pub fn segment_count(&self) -> u32 {
        self.duration_ms.div_ceil(SEGMENT_MS).max(1)
    }

    /// The 1-based segment containing `position_ms`.
    pub fn segment_at(&self, position_ms: u32) -> u32 {
        (position_ms / SEGMENT_MS + 1).min(self.segment_count())
    }

    /// Fetches the given segments (skipping ones already loaded) concurrently, plus the
    /// command danmaku on the first call, then filters everything loaded so far.
    pub async fn load(&self, indices: &[u32]) -> Result<DanmakuOutput> {
        let missing: Vec<u32> = {
            let l = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
            indices.iter().copied().filter(|i| !l.segments.contains_key(i)).collect()
        };
        let need_commands = self.loaded.lock().unwrap_or_else(|e| e.into_inner()).commands.is_none();

        let segments = join_all(missing.iter().map(|&i| self.api.segment(self.cid, i)));
        let commands = async { if need_commands { Some(self.api.view(self.cid).await) } else { None } };
        let (segments, commands) = tokio::join!(segments, commands);

        let mut l = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        for (i, seg) in missing.iter().zip(segments) {
            l.segments.insert(*i, seg?);
        }
        if let Some(view) = commands {
            // Command danmaku are optional; a failure here must not hide regular danmaku.
            l.commands = Some(view.map(|v| v.commands).unwrap_or_default());
        }
        let all: Vec<Danmaku> = l.segments.values().flatten().cloned().collect();
        let commands = l.commands.clone().unwrap_or_default();
        drop(l);
        Ok(process(all, commands, self.duration_ms, &self.opts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rubi_api::DanmakuView;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Default)]
    struct Fake {
        calls: AtomicU32,
    }

    #[async_trait]
    impl DanmakuApi for Fake {
        async fn segment(&self, _: u64, index: u32) -> rubi_api::Result<Vec<Danmaku>> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            let at = (index - 1) * SEGMENT_MS + 1000;
            Ok(vec![Danmaku {
                id: u64::from(index),
                progress_ms: at,
                mode: 1,
                content: format!("seg{index}"),
                ..Default::default()
            }])
        }

        async fn view(&self, _: u64) -> rubi_api::Result<DanmakuView> {
            Ok(DanmakuView { segment_count: 0, commands: vec![CommandDanmaku::default()] })
        }

        async fn send(&self, _: &rubi_api::SendDanmaku) -> rubi_api::Result<u64> {
            Ok(1)
        }
    }

    #[tokio::test]
    async fn loads_incrementally_and_skips_loaded_segments() {
        let fake = Arc::new(Fake::default());
        let s = DanmakuSession::open(
            fake.clone(),
            1,
            20 * 60 * 1000,
            DanmakuOptions { drop_commands: true, ..Default::default() },
        );
        assert_eq!(s.segment_count(), 4);
        assert_eq!(s.segment_at(7 * 60 * 1000), 2);

        let first = s.load(&[2]).await.unwrap();
        assert_eq!(first.kept.len(), 1);
        let more = s.load(&[1, 2, 3]).await.unwrap();
        assert_eq!(more.kept.len(), 3);
        assert_eq!(fake.calls.load(Ordering::Relaxed), 3, "segment 2 was not fetched twice");
        assert_eq!(more.removed[&rubi_filter::Category::CommandDanmaku], 1);
    }
}
