//! Offline stress tests of the services: nothing here reaches the network. Run with:
//! `cargo test -p rubi-core --test stress --release -- --ignored --nocapture`

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use rubi_core::{PlaylistMode, Services, Session};

/// A data directory of its own, gone when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("rubi-core-stress-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Fails rather than hangs when `f` deadlocks.
fn within(limit: Duration, f: impl FnOnce() + Send + 'static) {
    let (done, finished) = mpsc::channel();
    let worker = thread::spawn(move || {
        f();
        let _ = done.send(());
    });
    match finished.recv_timeout(limit) {
        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => worker.join().unwrap(),
        Err(mpsc::RecvTimeoutError::Timeout) => panic!("still running after {limit:?}: deadlock?"),
    }
}

#[test]
#[ignore = "stress"]
fn settings_patches_lose_nothing() {
    let dir = TempDir::new("settings");
    let app = Arc::new(Services::open(&dir.0, Session::default()).unwrap());
    let shared = app.clone();
    within(Duration::from_secs(120), move || {
        let app = shared;
        // Each writer owns one key; all of them must end on their own last value.
        let keys = ["preferred_quality", "candidate_ttl_s", "min_source_interval_s"];
        let writers: Vec<_> = keys
            .iter()
            .map(|&key| {
                let app = app.clone();
                thread::spawn(move || {
                    for n in 1..=300 {
                        app.preferences.patch_settings(&format!("{{\"{key}\": {n}}}")).unwrap();
                    }
                })
            })
            .collect();
        let flags: Vec<_> = (0..2)
            .map(|_| {
                let app = app.clone();
                thread::spawn(move || {
                    for i in 0..300 {
                        app.preferences.patch_settings(&format!("{{\"auto_pip\": {}}}", i % 2 == 1)).unwrap();
                        assert!(app.preferences.patch_settings("{\"preferred_quality\": \"x\"}").is_err());
                        assert!(app.preferences.patch_settings("[1]").is_err());
                        let _ = app.preferences.settings();
                    }
                })
            })
            .collect();
        writers.into_iter().chain(flags).for_each(|w| w.join().unwrap());
    });
    let s = app.preferences.settings();
    assert_eq!(s.playback.preferred_quality, 300);
    assert_eq!(s.feed.candidate_ttl_s, 300);
    assert_eq!(s.feed.min_source_interval_s, 300);
    assert!(s.playback.auto_pip, "ended on true");
    drop(app);
    let reopened = Services::open(&dir.0, Session::default()).unwrap();
    assert_eq!(reopened.preferences.settings(), s, "the last state is what was stored");
}

#[test]
#[ignore = "stress"]
fn one_name_one_list() {
    let dir = TempDir::new("names");
    let app = Arc::new(Services::open(&dir.0, Session::default()).unwrap());
    let shared = app.clone();
    within(Duration::from_secs(120), move || {
        let workers: Vec<_> = (0..12)
            .map(|_| {
                let app = shared.clone();
                thread::spawn(move || (0..50).filter(|i| app.playlists.create(&format!("同名{i}")).is_ok()).count())
            })
            .collect();
        let made: usize = workers.into_iter().map(|w| w.join().unwrap()).sum();
        assert_eq!(made, 50, "each name is taken once");
    });
    assert_eq!(app.playlists.lists().unwrap().len(), 51, "默认 and the 50");
}

#[test]
#[ignore = "stress"]
fn lists_queue_and_feed_do_not_block_each_other() {
    let dir = TempDir::new("mixed");
    let app = Arc::new(Services::open(&dir.0, Session::default()).unwrap());
    let shared = app.clone();
    let started = Instant::now();
    within(Duration::from_secs(180), move || {
        let app = shared;
        let mut workers = Vec::new();
        for w in 0..4 {
            let app = app.clone();
            workers.push(thread::spawn(move || {
                for i in 0..150 {
                    let id = app.playlists.create(&format!("列表{w}-{i}")).unwrap();
                    app.playlists.rename(id, &format!("改名{w}-{i}")).unwrap();
                    app.play_queue.play(id);
                    assert!(app.play_queue.first(id).unwrap().is_none());
                    app.playlists.delete(id).unwrap();
                }
            }));
        }
        for _ in 0..4 {
            let app = app.clone();
            workers.push(thread::spawn(move || {
                for i in 0..500 {
                    app.play_queue.playing().unwrap();
                    assert!(app.play_queue.next_after("BVnone").unwrap().is_none());
                    app.listening.items().unwrap();
                    assert!(app.listening.current().is_none());
                    assert_eq!(app.listening.related_left(), None);
                    app.listening.stop();
                    app.play_queue.drop_temp();
                    let mode = if i % 2 == 0 { PlaylistMode::Shuffle } else { PlaylistMode::LoopAll };
                    app.playlists.set_mode(mode).unwrap();
                }
            }));
        }
        for _ in 0..4 {
            let app = app.clone();
            workers.push(thread::spawn(move || {
                for _ in 0..500 {
                    app.home.new_session();
                    assert!(app.home.next_page(20).unwrap().is_empty(), "nothing was fetched");
                }
            }));
        }
        workers.into_iter().for_each(|w| w.join().unwrap());
    });
    eprintln!("mixed load finished in {:?}", started.elapsed());
    assert_eq!(app.playlists.lists().unwrap().len(), 1, "only 默认 is left");
    assert!(app.playlists.delete(rubi_core::DEFAULT_PLAYLIST).is_err(), "默认 stays");
    assert_eq!(app.play_queue.playing().unwrap().0, rubi_core::DEFAULT_PLAYLIST, "a deleted list is not left active");
}

/// Dropped services must close the database: on Windows an open file blocks removing the folder.
#[test]
#[ignore = "stress"]
fn opening_again_and_again_leaves_nothing_open() {
    let dir = TempDir::new("reopen");
    let started = Instant::now();
    for i in 0..60 {
        let app = Services::open(&dir.0, Session::default()).unwrap();
        app.preferences.patch_settings(&format!("{{\"preferred_quality\": {i}}}")).unwrap();
        drop(app);
    }
    eprintln!("60 opens in {:?}", started.elapsed());
    let app = Services::open(&dir.0, Session::default()).unwrap();
    assert_eq!(app.preferences.settings().playback.preferred_quality, 59);
    drop(app);
    std::fs::remove_dir_all(&dir.0).expect("no file is left open");
}
