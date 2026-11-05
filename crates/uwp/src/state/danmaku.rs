//! Where each danmaku sits at a moment of the video. The core places them on tracks and in
//! time; this turns that into a position in the picture, frame by frame.

use rubi_core::{Lane, Placement};
use rubi_model::Danmaku;

/// Time a scrolling danmaku takes to cross the picture.
pub const SCROLL_MS: u32 = 9_000;
/// Time a top or bottom danmaku stays.
pub const FIXED_MS: u32 = 4_000;
pub const FONT_SIZE: f64 = 22.0;
const TRACK_HEIGHT: f64 = 30.0;
/// Danmaku keep to the upper part of the picture.
const USED_HEIGHT: f64 = 0.6;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub text: String,
    /// `0xRRGGBB`.
    pub color: u32,
    pub lane: Lane,
    pub track: usize,
    pub start_ms: u32,
    pub width: f64,
}

/// Tracks that fit a picture `height` tall.
pub fn tracks(height: f64) -> u32 {
    ((height * USED_HEIGHT / TRACK_HEIGHT).floor() as u32).max(1)
}

/// The danmaku with their placements, earliest first.
pub fn items(danmaku: &[Danmaku], placements: &[Placement]) -> Vec<Item> {
    let mut items: Vec<Item> = placements
        .iter()
        .filter_map(|p| {
            let d = danmaku.iter().find(|d| d.id == p.id)?;
            Some(Item {
                text: d.content.clone(),
                color: d.color,
                lane: p.lane,
                track: p.track,
                start_ms: p.start_ms,
                width: p.width as f64,
            })
        })
        .collect();
    items.sort_by_key(|i| i.start_ms);
    items
}

/// The items on screen at `now_ms`, with where each one sits.
pub fn on_screen(items: &[Item], now_ms: u32, width: f64, height: f64) -> impl Iterator<Item = (usize, (f64, f64))> {
    let oldest = now_ms.saturating_sub(SCROLL_MS.max(FIXED_MS));
    let first = items.partition_point(|i| i.start_ms < oldest);
    items[first..]
        .iter()
        .enumerate()
        .take_while(move |(_, i)| i.start_ms <= now_ms)
        .filter_map(move |(at, i)| position(i, now_ms, width, height).map(|p| (first + at, p)))
}

fn position(item: &Item, now_ms: u32, width: f64, height: f64) -> Option<(f64, f64)> {
    let age = now_ms.checked_sub(item.start_ms)? as f64;
    let row = item.track as f64 * TRACK_HEIGHT;
    match item.lane {
        Lane::Scroll => {
            let done = age / SCROLL_MS as f64;
            (done <= 1.0).then(|| (width - (width + item.width) * done, row))
        }
        Lane::Top => (age <= FIXED_MS as f64).then(|| ((width - item.width) / 2.0, row)),
        Lane::Bottom => (age <= FIXED_MS as f64).then(|| ((width - item.width) / 2.0, height - TRACK_HEIGHT - row)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(lane: Lane, start_ms: u32) -> Item {
        Item { text: "a".into(), color: 0, lane, track: 2, start_ms, width: 100.0 }
    }

    #[test]
    fn scrolling_crosses_the_picture() {
        let i = item(Lane::Scroll, 1_000);
        assert_eq!(position(&i, 1_000, 800.0, 450.0), Some((800.0, 60.0)));
        assert_eq!(position(&i, 10_000, 800.0, 450.0), Some((-100.0, 60.0)));
        assert_eq!(position(&i, 10_001, 800.0, 450.0), None);
        assert_eq!(position(&i, 999, 800.0, 450.0), None);
    }

    #[test]
    fn fixed_ones_stay_centred() {
        assert_eq!(position(&item(Lane::Top, 0), 1_000, 800.0, 450.0), Some((350.0, 60.0)));
        assert_eq!(position(&item(Lane::Bottom, 0), 1_000, 800.0, 450.0), Some((350.0, 360.0)));
        assert_eq!(position(&item(Lane::Top, 0), 4_001, 800.0, 450.0), None);
    }

    #[test]
    fn only_those_on_screen_come_back() {
        let list = [item(Lane::Scroll, 0), item(Lane::Scroll, 20_000), item(Lane::Scroll, 40_000)];
        let shown: Vec<usize> = on_screen(&list, 21_000, 800.0, 450.0).map(|(at, _)| at).collect();
        assert_eq!(shown, [1]);
    }

    #[test]
    fn tracks_fit_the_picture() {
        assert_eq!(tracks(450.0), 9);
        assert_eq!(tracks(1.0), 1);
    }
}
