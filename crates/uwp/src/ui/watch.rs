//! Reports how far the video on show has been watched, so it lands in 历史记录. A report that fails
//! is dropped: the next one carries the same news.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use crate::state::watch::{LEFT, PLAYING, STARTED, WatchClock};
use crate::ui::shell::shell;
use crate::ui::{player, post};

const SAMPLE_EVERY: Duration = Duration::from_secs(1);

struct Watching {
    bvid: String,
    cid: u64,
    run: u64,
    opened: Instant,
    clock: WatchClock,
}

thread_local! {
    static WATCHING: RefCell<Option<Watching>> = const { RefCell::new(None) };
}

fn send(bvid: String, cid: u64, played_s: u64, real_s: u64, play_type: u8) {
    let Some(backend) = shell().and_then(|s| s.backend.get().cloned()) else { return };
    let core = backend.clone();
    backend.spawn(async move {
        let _ = core.services.history.report_watch(&bvid, cid, played_s, real_s, play_type).await;
    });
}

/// Follows part `cid` of `bvid`, which has just started playing.
pub fn start(bvid: &str, cid: u64) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let run = player::run();
    WATCHING.with_borrow_mut(|w| {
        *w = Some(Watching { bvid: bvid.to_owned(), cid, run, opened: Instant::now(), clock: WatchClock::default() });
    });
    send(bvid.to_owned(), cid, 0, 0, STARTED);
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        while player::run() == run {
            tokio::time::sleep(SAMPLE_EVERY).await;
            post(&dispatcher, move || sample(run));
        }
    });
}

fn sample(run: u64) {
    let Some((playing, position_s)) = player::progress() else { return };
    let due = WATCHING.with_borrow_mut(|w| {
        let watching = w.as_mut().filter(|w| w.run == run)?;
        let real_s = watching.opened.elapsed().as_secs();
        let due = watching.clock.sample(playing, position_s, real_s);
        due.then(|| (watching.bvid.clone(), watching.cid, watching.clock.progress()))
    });
    if let Some((bvid, cid, (played_s, real_s))) = due {
        send(bvid, cid, played_s, real_s, PLAYING);
    }
}

/// The video is being left: says how far it got.
pub fn stop() {
    if let Some(left) = WATCHING.with_borrow_mut(Option::take) {
        let (played_s, real_s) = left.clock.progress();
        send(left.bvid, left.cid, played_s, real_s, LEFT);
    }
}
