//! 竖屏推流: the videos of the feed and where in them the page is. The opening video comes
//! first, then portrait videos from its 相关推荐, none twice.

use std::collections::HashSet;

/// Videos added at a time, at least.
pub const BATCH: usize = 6;
/// Videos looked back through for related ones.
pub const LOOK_BACK: usize = 3;
/// Fewer than this many left after the one shown: fetch more.
const AHEAD: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub bvid: String,
    pub title: String,
    pub uploader: String,
}

#[derive(Debug, Default)]
pub struct Feed {
    items: Vec<Item>,
    seen: HashSet<String>,
    at: usize,
}

impl Feed {
    pub fn start(first: Item) -> Self {
        let seen = HashSet::from([first.bvid.clone()]);
        Self { items: vec![first], seen, at: 0 }
    }

    pub fn current(&self) -> Option<&Item> {
        self.items.get(self.at)
    }

    /// Which one is on show, from 1.
    pub fn position(&self) -> usize {
        self.at + 1
    }

    /// Moves `by` videos on; `None` at either end.
    pub fn step(&mut self, by: isize) -> Option<&Item> {
        let to = self.at.checked_add_signed(by)?;
        if to >= self.items.len() {
            return None;
        }
        self.at = to;
        self.current()
    }

    /// Adds videos not seen before; returns how many.
    pub fn add(&mut self, more: impl IntoIterator<Item = Item>) -> usize {
        let before = self.items.len();
        for item in more {
            if self.seen.insert(item.bvid.clone()) {
                self.items.push(item);
            }
        }
        self.items.len() - before
    }

    pub fn wants_more(&self) -> bool {
        self.items.len() < self.at + 1 + AHEAD
    }

    /// The videos to take related ones from, latest first.
    pub fn sources(&self) -> Vec<String> {
        self.items.iter().rev().take(LOOK_BACK).map(|i| i.bvid.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(b: &str) -> Item {
        Item { bvid: b.into(), title: String::new(), uploader: String::new() }
    }

    #[test]
    fn walks_the_videos_and_stops_at_the_ends() {
        let mut feed = Feed::start(item("a"));
        assert!(feed.step(-1).is_none());
        assert!(feed.step(1).is_none());
        feed.add([item("b"), item("c")]);
        assert_eq!(feed.step(1).unwrap().bvid, "b");
        assert_eq!(feed.position(), 2);
        assert_eq!(feed.step(-1).unwrap().bvid, "a");
    }

    #[test]
    fn no_video_comes_twice() {
        let mut feed = Feed::start(item("a"));
        assert_eq!(feed.add([item("a"), item("b"), item("b")]), 1);
    }

    #[test]
    fn asks_for_more_near_the_end() {
        let mut feed = Feed::start(item("a"));
        assert!(feed.wants_more());
        feed.add([item("b"), item("c"), item("d"), item("e")]);
        assert!(!feed.wants_more());
        feed.step(1);
        feed.step(1);
        assert!(feed.wants_more());
        assert_eq!(feed.sources(), ["e", "d", "c"]);
    }
}
