//! The panes the user can drag: how wide they may get, and how the content lays out at a width.

pub const RAIL_MIN: f64 = 48.0;
const RAIL_MAX: f64 = 240.0;
/// From this wide the navigation has room for its labels.
const RAIL_LABELS_FROM: f64 = 110.0;
const SIDE_MIN: f64 = 220.0;
/// The video keeps at least this much beside the side pane: the system player's controls will
/// not get much narrower, and it would spill out of its column.
const MAIN_MIN: f64 = 480.0;
/// From this wide the side pane's rows are large, with the cover on top.
const SIDE_LARGE_FROM: f64 = 400.0;
/// From this wide the side pane has two columns: each still as wide as a large row.
const SIDE_TWO_FROM: f64 = 2.0 * SIDE_LARGE_FROM;
/// Height of a small row, and what a large one adds to its cover.
const SMALL_ROW: f64 = 84.0;
const LARGE_ROW_TEXT: f64 = 66.0;
/// Space inside a row around its content.
const ROW_MARGIN: f64 = 24.0;

pub fn rail_width(wanted: f64) -> f64 {
    wanted.clamp(RAIL_MIN, RAIL_MAX)
}

pub fn rail_has_labels(width: f64) -> bool {
    width >= RAIL_LABELS_FROM
}

/// The side pane's width within a page `page_width` wide.
pub fn side_width(wanted: f64, page_width: f64) -> f64 {
    wanted.min(page_width - MAIN_MIN).max(SIDE_MIN)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SideLayout {
    pub large: bool,
    pub columns: u32,
    pub cell_width: f64,
    pub cell_height: f64,
}

pub fn side_layout(width: f64) -> SideLayout {
    let large = width >= SIDE_LARGE_FROM;
    let columns = if width >= SIDE_TWO_FROM { 2 } else { 1 };
    let cell_width = (width / columns as f64).floor().max(1.0);
    let cell_height =
        if large { ((cell_width - ROW_MARGIN).max(1.0) * 9.0 / 16.0).floor() + LARGE_ROW_TEXT } else { SMALL_ROW };
    SideLayout { large, columns, cell_width, cell_height }
}

/// The widths the user dragged to; `None` keeps the default.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Widths {
    pub rail: Option<f64>,
    pub side: Option<f64>,
}

impl Widths {
    /// Reads what [`to_text`](Self::to_text) wrote; anything else is the defaults.
    pub fn parse(text: &str) -> Self {
        let mut parts = text.split_whitespace().map(|p| p.parse::<f64>().ok().filter(|w| *w > 0.0));
        Self { rail: parts.next().flatten(), side: parts.next().flatten() }
    }

    pub fn to_text(self) -> String {
        format!("{} {}", self.rail.unwrap_or(0.0), self.side.unwrap_or(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panes_keep_to_their_limits() {
        assert_eq!(rail_width(10.0), 48.0);
        assert_eq!(rail_width(500.0), 240.0);
        assert!(!rail_has_labels(109.0) && rail_has_labels(110.0));
        assert_eq!(side_width(900.0, 1000.0), 520.0);
        assert_eq!(side_width(100.0, 1000.0), 220.0);
        assert_eq!(side_width(300.0, 400.0), 220.0, "a very small page still shows the pane");
    }

    #[test]
    fn rows_follow_the_side_pane() {
        assert_eq!(side_layout(300.0), SideLayout { large: false, columns: 1, cell_width: 300.0, cell_height: 84.0 });
        assert!(!side_layout(390.0).large, "small rows up to 400");
        let large = side_layout(424.0);
        assert!(large.large);
        assert_eq!(large.cell_height, 225.0 + 66.0);
        assert_eq!(side_layout(780.0).columns, 1, "one column until two large rows fit");
        assert_eq!(side_layout(820.0).columns, 2);
        assert_eq!(side_layout(820.0).cell_width, 410.0);
    }

    #[test]
    fn widths_survive_a_round_trip() {
        let widths = Widths { rail: Some(120.0), side: None };
        assert_eq!(Widths::parse(&widths.to_text()), widths);
        assert_eq!(Widths::parse("nonsense"), Widths::default());
        assert_eq!(Widths::parse(""), Widths::default());
    }
}
