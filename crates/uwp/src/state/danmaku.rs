//! Where each danmaku sits at a moment of the video. The core places them on tracks and in
//! time; this turns that into a position in the picture, frame by frame.

use rubi_core::{Lane, Placement};
use rubi_model::Danmaku;

/// Time a scrolling danmaku takes to cross the picture at the usual speed.
pub const SCROLL_MS: u32 = 9_000;
/// Time a top or bottom danmaku stays.
pub const FIXED_MS: u32 = 4_000;
/// The usual text size; a track is a little taller.
pub const FONT_SIZE: f64 = 22.0;
/// A track's height at the usual text size.
const TRACK_HEIGHT: f64 = 30.0;

/// How danmaku show, as the viewer set them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// Share of the picture's height danmaku keep to.
    pub area: f64,
    /// 1 is the usual speed.
    pub speed: f64,
    pub size: f64,
    /// 0.2 to 1.
    pub opacity: f64,
    pub scroll: bool,
    pub top: bool,
    pub bottom: bool,
    /// Danmaku in another colour than white.
    pub colored: bool,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            area: 0.5,
            speed: 1.0,
            size: FONT_SIZE,
            opacity: 1.0,
            scroll: true,
            top: true,
            bottom: true,
            colored: true,
        }
    }
}

/// The choices the settings offer: (label, value).
pub const AREAS: [(&str, f64); 3] = [("1/4", 0.25), ("半屏", 0.5), ("满屏", 1.0)];
pub const SPEEDS: [(&str, f64); 4] = [("慢", 0.6), ("适中", 1.0), ("快", 1.5), ("极快", 2.0)];
pub const SIZES: [(&str, f64); 3] = [("小", 18.0), ("标准", FONT_SIZE), ("大", 26.0)];

impl Look {
    pub fn scroll_ms(&self) -> u32 {
        (f64::from(SCROLL_MS) / self.speed.clamp(0.3, 3.0)) as u32
    }

    fn track_height(&self) -> f64 {
        self.size * TRACK_HEIGHT / FONT_SIZE
    }

    /// Whether `danmaku` is of a kind on show (mode 4 bottom, 5 top, others scroll).
    pub fn shows(&self, danmaku: &Danmaku) -> bool {
        let lane_on = match danmaku.mode {
            4 => self.bottom,
            5 => self.top,
            _ => self.scroll,
        };
        lane_on && (self.colored || danmaku.color & 0xFF_FFFF == 0xFF_FFFF)
    }

    /// The kinds on show as bits, for the preferences file: scroll, top, bottom, coloured.
    pub fn kinds(&self) -> u8 {
        u8::from(self.scroll) | u8::from(self.top) << 1 | u8::from(self.bottom) << 2 | u8::from(self.colored) << 3
    }

    pub fn with_kinds(self, bits: u8) -> Self {
        Self { scroll: bits & 1 != 0, top: bits & 2 != 0, bottom: bits & 4 != 0, colored: bits & 8 != 0, ..self }
    }
}

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

/// Tracks that fit the share of a picture `height` tall that danmaku keep to.
pub fn tracks(height: f64, look: &Look) -> u32 {
    ((height * look.area / look.track_height()).floor() as u32).max(1)
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
pub fn on_screen(
    items: &[Item],
    now_ms: u32,
    width: f64,
    height: f64,
    look: Look,
) -> impl Iterator<Item = (usize, (f64, f64))> + '_ {
    let oldest = now_ms.saturating_sub(look.scroll_ms().max(FIXED_MS));
    let first = items.partition_point(|i| i.start_ms < oldest);
    items[first..]
        .iter()
        .enumerate()
        .take_while(move |(_, i)| i.start_ms <= now_ms)
        .filter_map(move |(at, i)| position(i, now_ms, width, height, &look).map(|p| (first + at, p)))
}

fn position(item: &Item, now_ms: u32, width: f64, height: f64, look: &Look) -> Option<(f64, f64)> {
    let age = now_ms.checked_sub(item.start_ms)? as f64;
    let track = look.track_height();
    let row = item.track as f64 * track;
    match item.lane {
        Lane::Scroll => {
            let done = age / f64::from(look.scroll_ms());
            (done <= 1.0).then_some((width - (width + item.width) * done, row))
        }
        Lane::Top => (age <= FIXED_MS as f64).then_some(((width - item.width) / 2.0, row)),
        Lane::Bottom => (age <= FIXED_MS as f64).then_some(((width - item.width) / 2.0, height - track - row)),
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
        let look = Look::default();
        assert_eq!(position(&i, 1_000, 800.0, 450.0, &look), Some((800.0, 60.0)));
        assert_eq!(position(&i, 10_000, 800.0, 450.0, &look), Some((-100.0, 60.0)));
        assert_eq!(position(&i, 10_001, 800.0, 450.0, &look), None);
        assert_eq!(position(&i, 999, 800.0, 450.0, &look), None);
        let fast = Look { speed: 2.0, ..look };
        assert_eq!(position(&i, 5_500, 800.0, 450.0, &fast), Some((-100.0, 60.0)), "twice as fast, half the time");
    }

    #[test]
    fn fixed_ones_stay_centred() {
        let look = Look::default();
        assert_eq!(position(&item(Lane::Top, 0), 1_000, 800.0, 450.0, &look), Some((350.0, 60.0)));
        assert_eq!(position(&item(Lane::Bottom, 0), 1_000, 800.0, 450.0, &look), Some((350.0, 360.0)));
        assert_eq!(position(&item(Lane::Top, 0), 4_001, 800.0, 450.0, &look), None);
    }

    #[test]
    fn only_those_on_screen_come_back() {
        let list = [item(Lane::Scroll, 0), item(Lane::Scroll, 20_000), item(Lane::Scroll, 40_000)];
        let shown: Vec<usize> = on_screen(&list, 21_000, 800.0, 450.0, Look::default()).map(|(at, _)| at).collect();
        assert_eq!(shown, [1]);
    }

    #[test]
    fn tracks_fit_the_picture() {
        let look = Look::default();
        assert_eq!(tracks(450.0, &look), 7);
        assert_eq!(tracks(450.0, &Look { area: 1.0, ..look }), 15);
        assert_eq!(tracks(1.0, &look), 1);
    }

    #[test]
    fn kinds_hide_what_is_off() {
        let dm = |mode: u8, color: u32| Danmaku { mode, color, ..Danmaku::default() };
        let look = Look { top: false, colored: false, ..Look::default() };
        assert!(look.shows(&dm(1, 0xFF_FFFF)));
        assert!(!look.shows(&dm(5, 0xFF_FFFF)), "top is off");
        assert!(!look.shows(&dm(1, 0xFF_0000)), "coloured is off");
        assert!(look.shows(&dm(4, 0xFF_FFFF)));
        assert_eq!(Look::default().with_kinds(look.kinds()), Look { top: false, colored: false, ..Look::default() });
    }
}
