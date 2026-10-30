//! Track assignment for scrolling and fixed danmaku. The renderer only draws what
//! this returns; danmaku that find no free track are dropped (anti-overlap).

use rubi_model::{Danmaku, DanmakuMode};

/// Measures rendered text width. The Android layer injects one backed by its Paint;
/// [`EstimateMeasurer`] is a font-free fallback.
pub trait TextMeasurer {
    fn width(&self, text: &str, font_size: f32) -> f32;
}

/// CJK and full-width characters count as 1 em, everything else as 0.55 em.
pub struct EstimateMeasurer;

impl TextMeasurer for EstimateMeasurer {
    fn width(&self, text: &str, font_size: f32) -> f32 {
        text.chars().map(|c| if c.len_utf8() >= 3 { 1.0 } else { 0.55 }).sum::<f32>() * font_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConfig {
    pub viewport_w: f32,
    pub font_size: f32,
    pub tracks: usize,
    /// Time a scrolling danmaku takes to cross the screen.
    pub scroll_ms: u32,
    /// Time a top / bottom danmaku stays on screen.
    pub fixed_ms: u32,
    /// Minimum horizontal gap between consecutive danmaku on one track.
    pub gap: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { viewport_w: 412.0, font_size: 14.0, tracks: 8, scroll_ms: 8_000, fixed_ms: 4_000, gap: 16.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lane {
    Scroll,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub id: u64,
    pub lane: Lane,
    pub track: usize,
    pub start_ms: u32,
    pub width: f32,
}

#[derive(Clone, Copy)]
struct Occupant {
    start_ms: u32,
    width: f32,
}

/// Lays out `dms` (any order). Advanced / code / BAS danmaku are not laid out here.
pub fn layout(dms: &[Danmaku], cfg: &LayoutConfig, measurer: &dyn TextMeasurer) -> Vec<Placement> {
    let mut sorted: Vec<&Danmaku> = dms.iter().collect();
    sorted.sort_by_key(|d| (d.progress_ms, d.id));

    let mut scroll: Vec<Option<Occupant>> = vec![None; cfg.tracks];
    let mut top: Vec<Option<Occupant>> = vec![None; cfg.tracks];
    let mut bottom: Vec<Option<Occupant>> = vec![None; cfg.tracks];
    let mut out = Vec::new();

    for d in sorted {
        let lane = match d.kind() {
            DanmakuMode::Scroll | DanmakuMode::Reverse => Lane::Scroll,
            DanmakuMode::Top => Lane::Top,
            DanmakuMode::Bottom => Lane::Bottom,
            _ => continue,
        };
        let width = measurer.width(&d.content, cfg.font_size);
        let t = d.progress_ms;
        let tracks = match lane {
            Lane::Scroll => &mut scroll,
            Lane::Top => &mut top,
            Lane::Bottom => &mut bottom,
        };
        let free = |o: &Occupant| match lane {
            Lane::Scroll => scroll_free(o, t, width, cfg),
            Lane::Top | Lane::Bottom => t >= o.start_ms + cfg.fixed_ms,
        };
        if let Some(track) = tracks.iter().position(|o| o.as_ref().is_none_or(free)) {
            tracks[track] = Some(Occupant { start_ms: t, width });
            out.push(Placement { id: d.id, lane, track, start_ms: t, width });
        }
    }
    out
}

/// A scrolling track is free for a new danmaku when the previous one has fully entered
/// the screen (plus a gap) and the new one will not catch up with it before it leaves.
fn scroll_free(prev: &Occupant, t: u32, width: f32, cfg: &LayoutConfig) -> bool {
    let dur = cfg.scroll_ms as f32;
    let elapsed = t.saturating_sub(prev.start_ms) as f32;
    let prev_speed = (cfg.viewport_w + prev.width) / dur;
    let entered = elapsed * prev_speed >= prev.width + cfg.gap;
    let new_speed = (cfg.viewport_w + width) / dur;
    let new_reaches_left = t as f32 + cfg.viewport_w / new_speed;
    let prev_tail_leaves = prev.start_ms as f32 + dur;
    entered && new_reaches_left >= prev_tail_leaves
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dm(id: u64, ms: u32, mode: u8, text: &str) -> Danmaku {
        Danmaku { id, progress_ms: ms, mode, content: text.into(), ..Default::default() }
    }

    #[test]
    fn simultaneous_danmaku_use_different_tracks() {
        let dms: Vec<Danmaku> = (0..3).map(|i| dm(i, 0, 1, "同一时刻")).collect();
        let p = layout(&dms, &LayoutConfig::default(), &EstimateMeasurer);
        let tracks: Vec<usize> = p.iter().map(|x| x.track).collect();
        assert_eq!(tracks, vec![0, 1, 2]);
    }

    #[test]
    fn track_is_reused_once_free_and_overflow_is_dropped() {
        let cfg = LayoutConfig { tracks: 1, ..Default::default() };
        let dms = vec![dm(1, 0, 1, "第一条"), dm(2, 100, 1, "太早"), dm(3, 4_000, 1, "之后")];
        let ids: Vec<u64> = layout(&dms, &cfg, &EstimateMeasurer).iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn fixed_lanes_wait_for_expiry_and_advanced_is_skipped() {
        let cfg = LayoutConfig { tracks: 1, ..Default::default() };
        let dms = vec![
            dm(1, 0, 5, "顶部"),
            dm(2, 1_000, 5, "顶部2"),
            dm(3, 4_000, 5, "顶部3"),
            dm(4, 0, 4, "底部"),
            dm(5, 0, 7, "高级"),
        ];
        let p = layout(&dms, &cfg, &EstimateMeasurer);
        let got: Vec<(u64, Lane)> = p.iter().map(|x| (x.id, x.lane)).collect();
        assert_eq!(got, vec![(1, Lane::Top), (4, Lane::Bottom), (3, Lane::Top)]);
    }
}
