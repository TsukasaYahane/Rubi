//! The video page: details, streams, danmaku and interactive videos.

use std::sync::Arc;

use rubi_api::SendDanmaku;
use rubi_core::{DanmakuSession, SteinSession};
use rubi_filter::danmaku::layout::{Lane, LayoutConfig, MeasuredWidths, layout};
use rubi_model::{Danmaku, VideoRef};

use crate::{
    FfiDanmaku, FfiDanmakuSet, FfiLane, FfiPlacement, FfiPlayback, FfiResumePoint, FfiSteinNode, FfiVideoCard,
    FfiVideoDetail, FfiVideoRef, Res, RubiCore,
};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// Frees what watching videos piled up: cached cards and comment roots. Call when the user
    /// leaves the video pages, or when the system asks for memory.
    pub fn release_caches(&self) {
        self.videos.forget_shown();
        self.comments.forget();
    }

    pub async fn video(&self, bvid: String) -> Res<FfiVideoCard> {
        Ok(FfiVideoCard::of(&self.videos.card(&bvid).await?, None))
    }

    /// Full details for the player page (always fresh, with tags).
    pub async fn video_detail(&self, bvid: String) -> Res<FfiVideoDetail> {
        Ok(FfiVideoDetail::of(&self.videos.detail(&bvid).await?))
    }

    /// The BV id to open for a video pointer (av numbers are looked up, short links followed).
    pub async fn resolve_video(&self, video: FfiVideoRef) -> Res<String> {
        let video = match video {
            FfiVideoRef::Bvid { bvid } => VideoRef::Bvid(bvid),
            FfiVideoRef::Aid { aid } => VideoRef::Aid(aid),
            FfiVideoRef::Short { url } => VideoRef::Short(url),
        };
        Ok(self.videos.resolve(video).await?)
    }

    /// Where the user stopped watching this video, when worth offering; `None` otherwise.
    pub async fn resume_point(&self, bvid: String) -> Option<FfiResumePoint> {
        self.videos.resume_point(&bvid).await.map(|r| FfiResumePoint { cid: r.cid, position_ms: r.position_ms })
    }

    /// Stream URLs for `qn` (capped to the qualities Rubi offers).
    pub async fn playback(&self, bvid: String, qn: u32) -> Res<FfiPlayback> {
        Ok(self.videos.playback(&bvid, qn).await?.into())
    }

    /// Stream URLs of one part (`cid`), e.g. a branch of an interactive video.
    pub async fn playback_part(&self, bvid: String, cid: u64, qn: u32) -> Res<FfiPlayback> {
        Ok(self.videos.playback_part(&bvid, cid, qn).await?.into())
    }

    /// The story of an interactive video starting at part `cid`; `None` for ordinary videos.
    pub async fn stein_open(&self, bvid: String, cid: u64) -> Res<Option<Arc<FfiStein>>> {
        Ok(self.stein.open(&bvid, cid).await?.map(|s| Arc::new(FfiStein(s))))
    }

    /// Posts a danmaku at `progress_ms` and returns it, ready to add to the shown set.
    /// `mode`: 1 scroll, 4 bottom, 5 top; `color` is 0xRRGGBB. Login required.
    pub async fn send_danmaku(
        &self,
        bvid: String,
        cid: u64,
        progress_ms: u32,
        text: String,
        mode: u8,
        color: u32,
    ) -> Res<FfiDanmaku> {
        let sent = self.danmaku.send(SendDanmaku { bvid, cid, progress_ms, text, mode, color }).await?;
        Ok(sent.into())
    }

    /// All danmaku at once. Prefer [`danmaku_session`](Self::danmaku_session) for playback.
    pub async fn danmaku(&self, cid: u64, duration_ms: u32) -> Res<FfiDanmakuSet> {
        Ok(self.danmaku.all(cid, duration_ms).await?.into())
    }

    /// A segment-by-segment loader: load the segment being watched first, then the rest.
    pub fn danmaku_session(&self, cid: u64, duration_ms: u32) -> Res<Arc<FfiDanmakuSession>> {
        Ok(Arc::new(FfiDanmakuSession(self.danmaku.session(cid, duration_ms)?)))
    }

    /// Track layout for a batch of danmaku. `widths[i]` is the rendered width of `items[i]`
    /// (same unit as `viewport_w`; missing ones are estimated). `scroll_ms` is how long a
    /// scrolling danmaku takes to cross the screen; the renderer must use the same value.
    pub fn layout_danmaku(
        &self,
        items: Vec<FfiDanmaku>,
        widths: Vec<f32>,
        viewport_w: f32,
        font_size: f32,
        tracks: u32,
        scroll_ms: u32,
    ) -> Vec<FfiPlacement> {
        let dms: Vec<Danmaku> = items
            .into_iter()
            .map(|d| Danmaku {
                id: d.id,
                progress_ms: d.progress_ms,
                mode: d.mode,
                content: d.content,
                ..Default::default()
            })
            .collect();
        let cfg = LayoutConfig {
            viewport_w,
            font_size,
            tracks: tracks.max(1) as usize,
            scroll_ms: scroll_ms.max(1000),
            ..LayoutConfig::default()
        };
        let measured = MeasuredWidths(dms.iter().map(|d| d.content.as_str()).zip(widths).collect());
        layout(&dms, &cfg, &measured)
            .into_iter()
            .map(|p| FfiPlacement {
                id: p.id,
                lane: match p.lane {
                    Lane::Scroll => FfiLane::Scroll,
                    Lane::Top => FfiLane::Top,
                    Lane::Bottom => FfiLane::Bottom,
                },
                track: p.track as u32,
                start_ms: p.start_ms,
                width: p.width,
            })
            .collect()
    }
}

/// An interactive video being played: its current node and hidden variables.
#[derive(uniffi::Object)]
pub struct FfiStein(SteinSession);

#[uniffi::export(async_runtime = "tokio")]
impl FfiStein {
    pub fn node(&self) -> FfiSteinNode {
        FfiSteinNode::of(&self.0.current(), self.0.shown_vars())
    }

    /// Takes the choice leading to `edge_id`; play the returned node's part next.
    pub async fn choose(&self, edge_id: u64) -> Res<FfiSteinNode> {
        let n = self.0.choose(edge_id).await?;
        Ok(FfiSteinNode::of(&n, self.0.shown_vars()))
    }
}

#[derive(uniffi::Object)]
pub struct FfiDanmakuSession(DanmakuSession);

#[uniffi::export(async_runtime = "tokio")]
impl FfiDanmakuSession {
    pub fn segment_count(&self) -> u32 {
        self.0.segment_count()
    }

    /// The 1-based segment containing `position_ms`.
    pub fn segment_at(&self, position_ms: u32) -> u32 {
        self.0.segment_at(position_ms)
    }

    /// Loads `segments` (concurrently, skipping loaded ones) and returns the filtered
    /// danmaku over everything loaded so far.
    pub async fn load(&self, segments: Vec<u32>) -> Res<FfiDanmakuSet> {
        Ok(self.0.load(&segments).await?.into())
    }
}

impl From<rubi_filter::danmaku::DanmakuOutput> for FfiDanmakuSet {
    fn from(out: rubi_filter::danmaku::DanmakuOutput) -> Self {
        let mut removed: Vec<String> = out.removed.iter().map(|(c, n)| format!("{} {n}", c.label())).collect();
        removed.sort();
        Self {
            items: out.kept.into_iter().map(Into::into).collect(),
            jump_marks: out.jump_marks.into_iter().map(Into::into).collect(),
            removed,
        }
    }
}
