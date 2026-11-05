//! The three window sizes: the same lines as Android's window size classes.

const MEDIUM_MIN: f64 = 600.0;
const WIDE_MIN: f64 = 840.0;

/// Space around and between the cards of the video grid; the gap sits inside each cell.
const GRID_PADDING: f64 = 16.0;
const GRID_GAP: f64 = 14.0;
/// Room under a card's 16:9 cover: two lines of title, the meta line, and the gap to the next row.
const CARD_TEXT_HEIGHT: f64 = 86.0;
/// Width a wide window's cards keep to.
const WIDE_CARD_WIDTH: f64 = 280.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowClass {
    Narrow,
    Medium,
    Wide,
}

impl WindowClass {
    pub fn of(width: f64) -> Self {
        if width >= WIDE_MIN {
            Self::Wide
        } else if width >= MEDIUM_MIN {
            Self::Medium
        } else {
            Self::Narrow
        }
    }

    /// Columns of the video grid `list_width` wide: more on a big window, so cards stay the
    /// size they are meant to be instead of growing with it.
    pub fn columns(self, list_width: f64) -> u32 {
        match self {
            Self::Narrow => 2,
            Self::Medium => 3,
            Self::Wide => ((list_width / WIDE_CARD_WIDTH) as u32).clamp(4, 8),
        }
    }

    /// Width of the side navigation; none on a narrow window, which has the bottom bar.
    pub fn rail_width(self) -> f64 {
        match self {
            Self::Narrow => 0.0,
            Self::Medium => 48.0,
            Self::Wide => 188.0,
        }
    }
}

/// A video grid's cell in a list `list_width` wide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub width: f64,
    pub height: f64,
    pub cover_height: f64,
}

pub fn grid_cell(list_width: f64, columns: u32) -> Cell {
    let usable = (list_width - GRID_PADDING).max(1.0);
    let width = (usable / columns.max(1) as f64).floor();
    let cover_height = ((width - GRID_GAP).max(1.0) * 9.0 / 16.0).floor();
    Cell { width, height: cover_height + CARD_TEXT_HEIGHT, cover_height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_change_at_the_android_lines() {
        assert_eq!(WindowClass::of(599.0), WindowClass::Narrow);
        assert_eq!(WindowClass::of(600.0), WindowClass::Medium);
        assert_eq!(WindowClass::of(839.0), WindowClass::Medium);
        assert_eq!(WindowClass::of(840.0), WindowClass::Wide);
    }

    #[test]
    fn big_windows_get_more_columns() {
        assert_eq!(WindowClass::Wide.columns(996.0), 4);
        assert_eq!(WindowClass::Wide.columns(1728.0), 6);
        assert_eq!(WindowClass::Wide.columns(4000.0), 8);
        assert_eq!(WindowClass::Narrow.columns(500.0), 2);
    }

    #[test]
    fn cells_fill_the_row() {
        let cell = grid_cell(1016.0, 4);
        assert_eq!(cell.width, 250.0);
        assert_eq!(cell.cover_height, 132.0);
        assert_eq!(cell.height, 218.0);
        assert!(grid_cell(0.0, 0).width >= 1.0);
    }
}
