//! A comment thread as the list shows it: which rows, the fishbone lines each draws, what the
//! thread's foot offers, which branches are folded, and where a new reply goes. No XAML here.

use std::collections::HashSet;

use rubi_core::ThreadView;
use rubi_model::{Comment, ThreadRow, without_reply_prefix};

/// Deepest level drawn apart; deeper replies line up with it.
pub const DEEPEST: u32 = 6;
/// Replies one press of 展开 adds.
pub const STEP: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The replies on show so far.
    Open,
    /// The first comment alone.
    Collapsed,
    /// The whole thread is being read.
    Loading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Shows the replies again after a collapse.
    Expand,
    /// Shows [`STEP`] more replies, reading the whole thread first if need be.
    More,
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Foot {
    None,
    Loading,
    Acts(Vec<(Act, String)>),
}

/// A reply's own answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    None,
    /// Shown; this many below it.
    Open(usize),
    /// Folded away; this many below it.
    Folded(usize),
}

/// How a row draws its place in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    /// 0 for the first comment, then 1.. up to [`DEEPEST`].
    pub depth: u32,
    /// A later sibling follows on screen: the row's own line runs on to the bottom.
    pub later: bool,
    /// The levels above it whose lines pass by this row.
    pub through: Vec<u32>,
    pub branch: Branch,
}

pub struct Thread {
    pub view: ThreadView,
    pub mode: Mode,
    /// The whole thread has been read, not just the page's preview.
    pub whole: bool,
    /// Replies shown, in the tree's order.
    pub limit: usize,
    /// Replies whose answers are folded away.
    pub folded: HashSet<u64>,
}

impl Thread {
    pub fn new(view: ThreadView) -> Self {
        let limit = view.rows.len().saturating_sub(1);
        Self { view, mode: Mode::Open, whole: false, limit, folded: HashSet::new() }
    }

    pub fn root(&self) -> u64 {
        self.view.root_rpid
    }

    /// The rows within the limit, as indexes into the view's rows.
    fn within(&self) -> std::ops::Range<usize> {
        let replies =
            if self.mode == Mode::Collapsed { 0 } else { self.limit.min(self.view.rows.len().saturating_sub(1)) };
        0..(1 + replies).min(self.view.rows.len())
    }

    /// The rows on show, as indexes into the view's rows with their shapes.
    pub fn shown(&self) -> Vec<(usize, Shape)> {
        let rows = &self.view.rows;
        let within = self.within();
        // Rows past the folded ones, and how many each folded or open row has under it.
        let mut visible = Vec::new();
        let mut skip_below: Option<u32> = None;
        for at in within.clone() {
            let depth = rows[at].0.depth;
            if let Some(level) = skip_below {
                if depth > level {
                    continue;
                }
                skip_below = None;
            }
            visible.push(at);
            if depth > 0 && self.folded.contains(&rows[at].0.rpid) {
                skip_below = Some(depth);
            }
        }
        let under = |at: usize| rows[at + 1..within.end].iter().take_while(|(r, _)| r.depth > rows[at].0.depth).count();
        let depths: Vec<u32> = visible.iter().map(|at| rows[*at].0.depth.min(DEEPEST)).collect();
        let mut open = [false; DEEPEST as usize + 1];
        visible
            .iter()
            .enumerate()
            .map(|(i, at)| {
                let depth = depths[i];
                let later = depth > 0 && depths[i + 1..].iter().find(|d| **d <= depth).is_some_and(|d| *d == depth);
                let through = (1..depth).filter(|level| open[*level as usize]).collect();
                if depth > 0 {
                    open[depth as usize] = later;
                }
                let below = under(*at);
                let branch = match (depth, below) {
                    (0, _) | (_, 0) => Branch::None,
                    _ if self.folded.contains(&rows[*at].0.rpid) => Branch::Folded(below),
                    _ => Branch::Open(below),
                };
                (*at, Shape { depth, later, through, branch })
            })
            .collect()
    }

    /// Replies not on show for the limit: loaded past it, or not read yet.
    pub fn remaining(&self) -> usize {
        let total = (self.view.total_replies as usize).max(self.view.rows.len().saturating_sub(1));
        total.saturating_sub(self.limit.min(total))
    }

    pub fn foot(&self) -> Foot {
        let total = self.view.total_replies;
        match self.mode {
            Mode::Loading => Foot::Loading,
            Mode::Collapsed if total > 0 => Foot::Acts(vec![(Act::Expand, format!("展开 {total} 条回复"))]),
            Mode::Collapsed => Foot::None,
            Mode::Open => {
                let mut acts = Vec::new();
                let remaining = self.remaining();
                if remaining > 0 {
                    acts.push((Act::More, format!("再展开 {} 条 · 还剩 {remaining}", STEP.min(remaining))));
                }
                if self.within().len() > 1 {
                    acts.push((Act::Collapse, "折叠".to_owned()));
                }
                if acts.is_empty() { Foot::None } else { Foot::Acts(acts) }
            }
        }
    }

    /// Whether 展开 must read the whole thread first.
    pub fn needs_reading(&self) -> bool {
        !self.whole && self.view.rows.len().saturating_sub(1) < self.limit + STEP && self.remaining() > 0
    }

    /// Shows [`STEP`] more replies of what is loaded.
    pub fn more(&mut self) {
        self.limit += STEP;
        self.mode = Mode::Open;
    }

    /// The whole thread as read: keeps what was on show, and [`STEP`] more.
    pub fn read_whole(&mut self, view: ThreadView) {
        self.view = view;
        self.whole = true;
        self.more();
    }

    /// Folds a reply's answers away, or shows them again.
    pub fn toggle_branch(&mut self, rpid: u64) {
        if !self.folded.remove(&rpid) {
            self.folded.insert(rpid);
        }
    }

    /// Puts `reply` to `target` in the tree, as its last answer, and shows it. False when
    /// `target` is not among the rows.
    pub fn add_reply(&mut self, target: u64, reply: Comment) -> bool {
        let rows = &mut self.view.rows;
        let Some(at) = rows.iter().position(|(row, _)| row.rpid == target) else { return false };
        let depth = rows[at].0.depth;
        let end = at + 1 + rows[at + 1..].iter().take_while(|(row, _)| row.depth > depth).count();
        if let Some(last) = rows[at + 1..end].iter_mut().rev().find(|(row, _)| row.depth == depth + 1) {
            last.0.has_later_sibling = true;
        }
        let reply_to = (depth >= 1).then(|| rows[at].1.name.clone());
        let row = ThreadRow { rpid: reply.rpid, depth: depth + 1, has_later_sibling: false, reply_to };
        rows.insert(end, (row, reply));
        self.view.total_replies += 1;
        // Shown however far the limit reached: the rows up to it count.
        self.limit = self.limit.max(end);
        self.folded.remove(&target);
        if self.mode == Mode::Collapsed {
            self.mode = Mode::Open;
        }
        true
    }
}

/// A row's text: who it answers, then what it says without B站's own lead-in.
pub fn body(row: &ThreadRow, comment: &Comment) -> String {
    match &row.reply_to {
        Some(name) => format!("回复 @{name}：{}", without_reply_prefix(&comment.text)),
        None => comment.text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comment(rpid: u64, name: &str) -> Comment {
        Comment { rpid, name: name.to_owned(), text: format!("说 {rpid}"), ..Comment::default() }
    }

    fn row(rpid: u64, depth: u32) -> ThreadRow {
        ThreadRow { rpid, depth, has_later_sibling: false, reply_to: None }
    }

    /// 1 ─ 2 ─ 3
    ///   └ 4
    fn thread() -> Thread {
        let rows = vec![
            (row(1, 0), comment(1, "甲")),
            (row(2, 1), comment(2, "乙")),
            (row(3, 2), comment(3, "丙")),
            (row(4, 1), comment(4, "丁")),
        ];
        Thread::new(ThreadView { root_rpid: 1, rows, hidden: 7, total_replies: 10 })
    }

    fn shapes(t: &Thread) -> Vec<(u64, Shape)> {
        t.shown().into_iter().map(|(at, s)| (t.view.rows[at].0.rpid, s)).collect()
    }

    #[test]
    fn lines_pass_by_while_a_branch_goes_on() {
        let s = shapes(&thread());
        assert_eq!(s[0].1, Shape { depth: 0, later: false, through: vec![], branch: Branch::None });
        assert_eq!(s[1].1, Shape { depth: 1, later: true, through: vec![], branch: Branch::Open(1) });
        assert_eq!(s[2].1, Shape { depth: 2, later: false, through: vec![1], branch: Branch::None }, "丁 follows 乙");
        assert_eq!(s[3].1, Shape { depth: 1, later: false, through: vec![], branch: Branch::None });
    }

    #[test]
    fn a_folded_branch_hides_its_answers() {
        let mut t = thread();
        t.toggle_branch(2);
        let s = shapes(&t);
        assert_eq!(s.iter().map(|(r, _)| *r).collect::<Vec<_>>(), [1, 2, 4]);
        assert_eq!(s[1].1.branch, Branch::Folded(1));
        t.toggle_branch(2);
        assert_eq!(shapes(&t).len(), 4);
    }

    #[test]
    fn replies_open_four_at_a_time() {
        let mut t = thread();
        assert_eq!(t.remaining(), 7);
        assert_eq!(
            t.foot(),
            Foot::Acts(vec![(Act::More, "再展开 4 条 · 还剩 7".into()), (Act::Collapse, "折叠".into())])
        );
        assert!(t.needs_reading(), "the page brought three of ten");
        let mut rows = t.view.rows.clone();
        for rpid in 5..=11 {
            rows.push((row(rpid, 1), comment(rpid, "某")));
        }
        t.read_whole(ThreadView { root_rpid: 1, rows, hidden: 0, total_replies: 10 });
        assert_eq!(t.shown().len(), 1 + 7);
        assert_eq!(t.remaining(), 3);
        assert!(!t.needs_reading());
        t.more();
        assert_eq!(t.shown().len(), 1 + 10);
        assert_eq!(t.foot(), Foot::Acts(vec![(Act::Collapse, "折叠".into())]));
    }

    #[test]
    fn a_cut_branch_ends_its_line() {
        let mut t = thread();
        t.limit = 2;
        let s = shapes(&t);
        assert_eq!(s.len(), 3);
        assert!(!s[1].1.later, "丁 is past the limit");
    }

    #[test]
    fn the_foot_follows_the_mode() {
        let mut t = thread();
        t.mode = Mode::Collapsed;
        assert_eq!(t.shown().len(), 1);
        assert_eq!(t.foot(), Foot::Acts(vec![(Act::Expand, "展开 10 条回复".into())]));
        t.mode = Mode::Loading;
        assert_eq!(t.foot(), Foot::Loading);
        let lone = Thread::new(ThreadView {
            root_rpid: 9,
            rows: vec![(row(9, 0), comment(9, "戊"))],
            ..ThreadView::default()
        });
        assert_eq!(lone.foot(), Foot::None);
    }

    #[test]
    fn a_reply_goes_last_under_what_it_answers() {
        let mut t = thread();
        t.toggle_branch(2);
        assert!(t.add_reply(2, comment(20, "我")));
        let order: Vec<u64> = t.view.rows.iter().map(|(r, _)| r.rpid).collect();
        assert_eq!(order, [1, 2, 3, 20, 4]);
        let (new, _) = &t.view.rows[3];
        assert_eq!((new.depth, new.reply_to.as_deref()), (2, Some("乙")));
        assert!(shapes(&t).iter().any(|(r, _)| *r == 20), "the branch opened to show it");
        assert_eq!(t.view.total_replies, 11);

        assert!(t.add_reply(1, comment(21, "我")));
        let (last, _) = t.view.rows.last().unwrap();
        assert_eq!((last.rpid, last.depth, last.reply_to.clone()), (21, 1, None));
        assert!(shapes(&t).iter().any(|(r, _)| *r == 21));
        assert!(!t.add_reply(99, comment(22, "我")));
    }

    #[test]
    fn a_reply_to_a_collapsed_thread_opens_it() {
        let mut t = thread();
        t.mode = Mode::Collapsed;
        t.add_reply(1, comment(20, "我"));
        assert_eq!(t.mode, Mode::Open);
    }

    #[test]
    fn the_body_names_who_is_answered_once() {
        let mut c = comment(5, "乙");
        c.text = "回复 @甲 :好的".into();
        let r = ThreadRow { rpid: 5, depth: 2, has_later_sibling: false, reply_to: Some("甲".into()) };
        assert_eq!(body(&r, &c), "回复 @甲：好的");
        assert_eq!(body(&row(5, 1), &comment(5, "乙")), "说 5");
    }
}
