//! Offline stress tests of the store. Run with:
//! `cargo test -p rubi-store --test stress --release -- --ignored --nocapture`

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use rubi_model::{FeedbackEvent, Signal, VideoCard};
use rubi_store::{
    CandidateRepo, DEFAULT_PLAYLIST, EventRepo, FilterLogEntry, FilterLogRepo, HousekeepingRepo, KvRepo, PlaylistRepo,
    SqliteStore, WatchedRepo,
};

const NOW: i64 = 1_800_000_000;

/// A database file in a directory of its own, gone when dropped.
struct TempDb(PathBuf);

impl TempDb {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("rubi-stress-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> PathBuf {
        self.0.join("rubi.db")
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn card(bvid: String) -> VideoCard {
    VideoCard { bvid, ..Default::default() }
}

#[test]
#[ignore = "stress"]
fn kv_read_modify_write_loses_nothing() {
    let db = TempDb::new("kv");
    let store = Arc::new(SqliteStore::open(db.path()).unwrap());
    let started = Instant::now();
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let store = store.clone();
            thread::spawn(move || {
                for _ in 0..500 {
                    store
                        .update_raw("n", &mut |v| {
                            Ok((v.and_then(|s| s.parse::<u64>().ok()).unwrap_or(0) + 1).to_string())
                        })
                        .unwrap();
                }
            })
        })
        .collect();
    workers.into_iter().for_each(|w| w.join().unwrap());
    assert_eq!(store.get_raw("n").unwrap().as_deref(), Some("4000"));
    eprintln!("kv: 4000 updates over 8 threads in {:?}", started.elapsed());
}

/// The app opens one store; a second connection to the file (a second process) must wait for
/// the first rather than fail.
#[test]
#[ignore = "stress"]
fn two_connections_to_one_file_do_not_fail() {
    let db = TempDb::new("two");
    let path = db.path();
    SqliteStore::open(&path).unwrap();
    let workers: Vec<_> = (0..2)
        .map(|w| {
            let path = path.clone();
            thread::spawn(move || {
                let store = SqliteStore::open(&path).unwrap();
                (0..300).filter(|i| store.put_raw(&format!("k{w}-{i}"), "v").is_err()).count()
            })
        })
        .collect();
    let failed: usize = workers.into_iter().map(|w| w.join().unwrap()).sum();
    assert_eq!(failed, 0, "writes failed while the other connection held the file");
}

#[test]
#[ignore = "stress"]
fn playlist_stays_whole_under_concurrent_edits() {
    let db = TempDb::new("list");
    let store = Arc::new(SqliteStore::open(db.path()).unwrap());
    let workers: Vec<_> = (0..6u32)
        .map(|w| {
            let store = store.clone();
            thread::spawn(move || {
                for i in 0..120u32 {
                    let bvid = format!("BV{w}x{i}");
                    assert!(store.add_to_playlist(DEFAULT_PLAYLIST, &card(bvid.clone())).unwrap());
                    store.move_in_playlist(DEFAULT_PLAYLIST, (i as usize * 7) % 50, (i as usize * 3) % 50).unwrap();
                    if i % 3 == 0 {
                        store.remove_from_playlist(DEFAULT_PLAYLIST, &bvid).unwrap();
                    }
                    store.playlist(DEFAULT_PLAYLIST).unwrap();
                }
            })
        })
        .collect();
    workers.into_iter().for_each(|w| w.join().unwrap());
    let items = store.playlist(DEFAULT_PLAYLIST).unwrap();
    let unique: HashSet<&str> = items.iter().map(|c| c.bvid.as_str()).collect();
    assert_eq!(items.len(), 6 * 80, "two thirds of the videos stay");
    assert_eq!(unique.len(), items.len(), "no repeats");
    // Positions stay usable: moving the last to the front really moves it.
    let last = items.last().unwrap().bvid.clone();
    store.move_in_playlist(DEFAULT_PLAYLIST, items.len() - 1, 0).unwrap();
    assert_eq!(store.playlist(DEFAULT_PLAYLIST).unwrap()[0].bvid, last);
}

#[test]
#[ignore = "stress"]
fn growing_tables_keep_to_their_limits() {
    let store = SqliteStore::in_memory().unwrap();
    let started = Instant::now();
    for i in 0..60_010 {
        store.append_event(&FeedbackEvent { signal: Signal::Click, item: None, at: NOW + i }).unwrap();
    }
    eprintln!("events: 60010 appended in {:?}", started.elapsed());
    store.prune(NOW + 60_010).unwrap();
    let kept = store.events_since(0).unwrap();
    assert_eq!(kept.len(), 50_000, "the newest 50000 stay");
    assert_eq!(kept.iter().map(|e| e.at).max(), Some(NOW + 60_009));

    let entry = |i: usize| FilterLogEntry {
        at: NOW,
        rule_id: format!("r{i}"),
        category: "ad".into(),
        title: "t".into(),
        source: "s".into(),
        reason: "x".into(),
    };
    for chunk in (0..6_000).collect::<Vec<_>>().chunks(100) {
        store.append_filter_log(&chunk.iter().map(|&i| entry(i)).collect::<Vec<_>>()).unwrap();
    }
    assert!(store.recent_filter_log(10_000).unwrap().len() <= 5_000);

    store.mark_watched("old", NOW - 366 * 86_400).unwrap();
    store.mark_watched("new", NOW).unwrap();
    store.prune(NOW).unwrap();
    assert!(!store.is_watched("old").unwrap());
    assert!(store.is_watched("new").unwrap());
}

#[test]
#[ignore = "stress"]
fn large_batches_stay_quick() {
    let store = SqliteStore::in_memory().unwrap();
    let cards: Vec<(VideoCard, f32)> = (0..20_000).map(|i| (card(format!("BV{i}")), 1.0)).collect();
    let started = Instant::now();
    store.put_candidates(&cards, 3600, NOW).unwrap();
    eprintln!("candidates: put 20000 in {:?}", started.elapsed());
    let started = Instant::now();
    assert_eq!(store.fresh_candidates(NOW).unwrap().len(), 20_000);
    eprintln!("candidates: read 20000 in {:?}", started.elapsed());
    assert_eq!(store.purge_expired(NOW + 3601).unwrap(), 20_000);

    for i in (0..40_000).step_by(2) {
        store.mark_watched(&format!("BV{i}"), NOW).unwrap();
    }
    let ids: Vec<String> = (0..40_000).map(|i| format!("BV{i}")).collect();
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let started = Instant::now();
    assert_eq!(store.watched_among(&refs).unwrap().len(), 20_000);
    eprintln!("watched: 40000 looked up in {:?}", started.elapsed());
}
