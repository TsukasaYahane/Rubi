//! The queue on play, if any: the video page follows it from one video to the next.

use std::cell::RefCell;

use windows_core::Result;

pub use crate::state::queue::Entry;
use crate::state::queue::Queue;
use crate::ui::player;

thread_local! {
    static QUEUE: RefCell<Option<Queue>> = const { RefCell::new(None) };
}

/// Plays `entries` one after another, from `at`.
pub fn start(name: &str, entries: Vec<Entry>, at: usize) -> Result<()> {
    let Some(queue) = Queue::new(name, entries, at) else { return Ok(()) };
    QUEUE.with_borrow_mut(|q| *q = Some(queue));
    play_current()
}

fn play_current() -> Result<()> {
    let Some(entry) = QUEUE.with_borrow(|q| q.as_ref().map(|q| q.current().clone())) else { return Ok(()) };
    player::show(&entry.bvid, &entry.title, &entry.owner)
}

/// A video page opens `bvid`: the queue goes on only if that is its own.
pub fn opening(bvid: &str) {
    QUEUE.with_borrow_mut(|q| {
        if q.as_ref().is_some_and(|q| !q.holds(bvid)) {
            *q = None;
        }
    });
}

/// Moves by `by` and plays; past either end nothing happens.
pub fn step(by: isize) {
    let moved = QUEUE.with_borrow_mut(|q| q.as_mut().is_some_and(|q| q.step(by).is_some()));
    if moved {
        let _ = play_current();
    }
}

/// Plays the entry at `at`.
pub fn go(at: usize) {
    let moved = QUEUE.with_borrow_mut(|q| q.as_mut().is_some_and(|q| q.go(at).is_some()));
    if moved {
        let _ = play_current();
    }
}

/// The video on play ended: the next one follows, and after the last nothing does.
pub fn ended() {
    step(1);
}

/// What the video page shows of the queue.
pub fn view() -> Option<Queue> {
    QUEUE.with_borrow(Clone::clone)
}
