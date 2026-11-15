//! Videos played one after another, such as 稍后再看's 全部播放.

/// A video in the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub bvid: String,
    pub title: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queue {
    name: String,
    entries: Vec<Entry>,
    at: usize,
}

impl Queue {
    /// Starts at `at`, kept within the list. None for an empty list.
    pub fn new(name: &str, entries: Vec<Entry>, at: usize) -> Option<Self> {
        let at = at.min(entries.len().checked_sub(1)?);
        Some(Self { name: name.to_owned(), entries, at })
    }

    pub fn current(&self) -> &Entry {
        &self.entries[self.at]
    }

    /// What the list is called.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn at(&self) -> usize {
        self.at
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Moves by `by`; None, staying put, past either end.
    pub fn step(&mut self, by: isize) -> Option<&Entry> {
        let at = self.at.checked_add_signed(by)?;
        self.go(at)
    }

    /// Moves to `at`; None, staying put, past the end.
    pub fn go(&mut self, at: usize) -> Option<&Entry> {
        if at >= self.entries.len() {
            return None;
        }
        self.at = at;
        Some(self.current())
    }

    /// Whether `bvid` is the one playing: anything else opened leaves the queue.
    pub fn holds(&self, bvid: &str) -> bool {
        self.current().bvid == bvid
    }

    pub fn has_previous(&self) -> bool {
        self.at > 0
    }

    pub fn has_next(&self) -> bool {
        self.at + 1 < self.entries.len()
    }

    /// The tab's name: `稍后再看 3/5`.
    pub fn label(&self) -> String {
        format!("{} {}/{}", self.name, self.at + 1, self.entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(count: usize) -> Vec<Entry> {
        (0..count).map(|n| Entry { bvid: format!("BV{n}"), title: format!("第 {n} 个"), owner: "猫".into() }).collect()
    }

    #[test]
    fn an_empty_list_makes_no_queue() {
        assert_eq!(Queue::new("稍后再看", Vec::new(), 0), None);
    }

    #[test]
    fn the_start_stays_within_the_list() {
        let queue = Queue::new("稍后再看", entries(3), 9).unwrap();
        assert_eq!(queue.at(), 2);
        assert_eq!(queue.label(), "稍后再看 3/3");
    }

    #[test]
    fn steps_stop_at_either_end() {
        let mut queue = Queue::new("稍后再看", entries(3), 0).unwrap();
        assert_eq!(queue.step(-1), None);
        assert!(!queue.has_previous());
        assert_eq!(queue.step(1).map(|e| e.bvid.clone()).as_deref(), Some("BV1"));
        assert_eq!(queue.step(1).map(|e| e.bvid.clone()).as_deref(), Some("BV2"));
        assert!(!queue.has_next());
        assert_eq!(queue.step(1), None);
        assert_eq!(queue.at(), 2);
        assert_eq!(queue.go(7), None);
        assert_eq!(queue.go(0).map(|e| e.bvid.clone()).as_deref(), Some("BV0"));
        assert!(queue.holds("BV0"));
        assert!(!queue.holds("BV1"));
    }
}
