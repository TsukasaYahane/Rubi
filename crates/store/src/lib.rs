//! Local persistence. Callers depend on the repository traits; [`SqliteStore`]
//! implements all of them over a single SQLite file.
//!
//! Every read-modify-write a caller needs (a play list item, a download record, a JSON value)
//! is one call here, done under the connection lock in a transaction, so two callers can never
//! interleave and lose an update.

mod schema;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;

use rubi_model::{Download, FeedbackEvent, Signal, VideoCard};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("stored data could not be decoded: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

/// One entry of the "被过滤内容" page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterLogEntry {
    pub at: i64,
    pub rule_id: String,
    pub category: String,
    pub title: String,
    pub source: String,
    pub reason: String,
}

pub trait CandidateRepo {
    fn put_candidates(&self, cards: &[(VideoCard, f32)], ttl_s: i64, now: i64) -> Result<()>;
    fn fresh_candidates(&self, now: i64) -> Result<Vec<(VideoCard, f32)>>;
    fn purge_expired(&self, now: i64) -> Result<usize>;
}

pub trait ExposureRepo {
    fn record_impressions(&self, bvids: &[String], now: i64) -> Result<()>;
    fn record_click(&self, bvid: &str) -> Result<()>;
    /// Impressions since the last click, per bvid.
    fn exposures_since_click(&self) -> Result<HashMap<String, u32>>;
}

pub trait EventRepo {
    fn append_event(&self, ev: &FeedbackEvent) -> Result<()>;
    fn events_since(&self, at: i64) -> Result<Vec<FeedbackEvent>>;
    /// Events since `at` whose signal is one of `kinds` (see [`signal_kind`]), oldest first.
    fn events_of_kinds_since(&self, at: i64, kinds: &[&str]) -> Result<Vec<FeedbackEvent>>;
    /// Replaces every event with `events` in one transaction.
    fn replace_events(&self, events: &[FeedbackEvent]) -> Result<()>;
}

pub trait WatchedRepo {
    fn mark_watched(&self, bvid: &str, now: i64) -> Result<()>;
    fn is_watched(&self, bvid: &str) -> Result<bool>;
    /// Which of `bvids` were watched, in one go.
    fn watched_among(&self, bvids: &[&str]) -> Result<HashSet<String>>;
}

pub trait FilterLogRepo {
    fn append_filter_log(&self, entries: &[FilterLogEntry]) -> Result<()>;
    fn recent_filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>>;
}

/// Keeps the tables that only grow (events, exposures, watched) within bounds.
pub trait HousekeepingRepo {
    fn prune(&self, now: i64) -> Result<()>;
}

/// Exposures not seen again for this long are forgotten.
const EXPOSURE_KEEP_S: i64 = 30 * 86_400;
/// Watched marks older than this are forgotten (the video may be offered again).
const WATCHED_KEEP_S: i64 = 365 * 86_400;
/// Most feedback events kept; the oldest go first.
const MAX_EVENTS: i64 = 50_000;
/// Most filter log entries kept.
const MAX_FILTER_LOG: i64 = 5_000;

/// A named play list and how many videos it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistInfo {
    pub id: i64,
    pub name: String,
    pub count: usize,
}

/// The id of 默认, the list that always exists.
pub const DEFAULT_PLAYLIST: i64 = 1;

/// The custom play lists, each in order.
pub trait PlaylistRepo {
    /// Every list, 默认 first, then in the order they were made.
    fn playlists(&self) -> Result<Vec<PlaylistInfo>>;
    /// Adds an empty list at the end; returns its id. Names are unique.
    fn create_playlist(&self, name: &str) -> Result<i64>;
    fn rename_playlist(&self, id: i64, name: &str) -> Result<()>;
    /// Removes a list and its items (默认 stays).
    fn delete_playlist(&self, id: i64) -> Result<()>;
    fn playlist(&self, id: i64) -> Result<Vec<VideoCard>>;
    /// Replaces a whole list (lists are short).
    fn set_playlist(&self, id: i64, items: &[VideoCard]) -> Result<()>;
    /// The lists that hold `bvid`.
    fn lists_with(&self, bvid: &str) -> Result<Vec<i64>>;
    /// Appends `card` to list `id` unless it is there; returns whether it was added.
    fn add_to_playlist(&self, id: i64, card: &VideoCard) -> Result<bool>;
    fn remove_from_playlist(&self, id: i64, bvid: &str) -> Result<()>;
    /// Moves the item at `from` to `to` (clamped); an out-of-range `from` changes nothing.
    fn move_in_playlist(&self, id: i64, from: usize, to: usize) -> Result<()>;
    /// Replaces every list with `lists` in one transaction: entry `default_at` fills 默认, the
    /// others are made anew. Nothing changes when any step fails.
    fn replace_playlists(&self, lists: &[(String, Vec<VideoCard>)], default_at: usize) -> Result<()>;
}

/// 离线缓存 records.
pub trait DownloadRepo {
    /// Newest first.
    fn downloads(&self) -> Result<Vec<Download>>;
    fn download(&self, bvid: &str) -> Result<Option<Download>>;
    /// Adds or replaces the record of `d.card.bvid`.
    fn put_download(&self, d: &Download) -> Result<()>;
    /// Changes the record of `bvid` in place, with no other write in between; false when
    /// there is none.
    fn update_download(&self, bvid: &str, f: &mut dyn FnMut(&mut Download)) -> Result<bool>;
    fn remove_download(&self, bvid: &str) -> Result<()>;
}

/// Typed key/value settings (rule table, weights, profile…).
pub trait KvRepo {
    fn get_raw(&self, key: &str) -> Result<Option<String>>;
    fn put_raw(&self, key: &str, value: &str) -> Result<()>;
    /// Reads, changes and writes `key` with no other write in between.
    fn update_raw(&self, key: &str, f: &mut dyn FnMut(Option<String>) -> Result<String>) -> Result<()>;

    fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>
    where
        Self: Sized,
    {
        Ok(self.get_raw(key)?.map(|s| serde_json::from_str(&s)).transpose()?)
    }

    fn put_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()>
    where
        Self: Sized,
    {
        self.put_raw(key, &serde_json::to_string(value)?)
    }

    /// [`update_raw`](Self::update_raw) for a JSON value (its default when missing); returns
    /// the value written.
    fn update_json<T: DeserializeOwned + Serialize + Default + Clone>(
        &self,
        key: &str,
        f: impl FnOnce(&mut T),
    ) -> Result<T>
    where
        Self: Sized,
    {
        let mut f = Some(f);
        let mut written = None;
        self.update_raw(key, &mut |old| {
            let mut v: T = old.map(|s| serde_json::from_str(&s)).transpose()?.unwrap_or_default();
            if let Some(f) = f.take() {
                f(&mut v);
            }
            let text = serde_json::to_string(&v)?;
            written = Some(v);
            Ok(text)
        })?;
        Ok(written.unwrap_or_default())
    }
}

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Self> {
        conn.set_prepared_statement_cache_capacity(48);
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        schema::migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn with<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut conn)
    }

    fn insert_items(tx: &rusqlite::Transaction<'_>, id: i64, items: &[VideoCard]) -> Result<()> {
        let mut stmt = tx
            .prepare_cached("INSERT OR IGNORE INTO playlist_items(list_id, pos, bvid, card) VALUES (?1, ?2, ?3, ?4)")?;
        for (pos, card) in items.iter().enumerate() {
            stmt.execute(params![id, pos as i64, card.bvid, serde_json::to_string(card)?])?;
        }
        Ok(())
    }
}

impl CandidateRepo for SqliteStore {
    fn put_candidates(&self, cards: &[(VideoCard, f32)], ttl_s: i64, now: i64) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            {
                let mut old_of = tx.prepare_cached("SELECT card FROM candidates WHERE bvid = ?1")?;
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO candidates(bvid, card, penalty, fetched_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(bvid) DO UPDATE SET card = excluded.card, penalty = min(penalty, excluded.penalty),
                       fetched_at = excluded.fetched_at, expires_at = excluded.expires_at",
                )?;
                for (card, penalty) in cards {
                    let mut merged = card.clone();
                    // Keep sources from earlier fetches so the topic factor sees every source.
                    if let Some(old) = old_of.query_row([&card.bvid], |r| r.get::<_, String>(0)).optional()? {
                        let old: VideoCard = serde_json::from_str(&old)?;
                        for s in old.sources {
                            merged.add_source(s);
                        }
                    }
                    stmt.execute(params![card.bvid, serde_json::to_string(&merged)?, penalty, now, now + ttl_s])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    fn fresh_candidates(&self, now: i64) -> Result<Vec<(VideoCard, f32)>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT card, penalty FROM candidates WHERE expires_at > ?1")?;
            let rows = stmt.query_map([now], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f32>(1)?)))?;
            rows.map(|r| {
                let (json, p) = r?;
                Ok((serde_json::from_str(&json)?, p))
            })
            .collect()
        })
    }

    fn purge_expired(&self, now: i64) -> Result<usize> {
        self.with(|c| Ok(c.prepare_cached("DELETE FROM candidates WHERE expires_at <= ?1")?.execute([now])?))
    }
}

impl ExposureRepo for SqliteStore {
    fn record_impressions(&self, bvids: &[String], now: i64) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO exposures(bvid, since_click, total, last_at) VALUES (?1, 1, 1, ?2)
                     ON CONFLICT(bvid) DO UPDATE SET since_click = since_click + 1, total = total + 1, last_at = excluded.last_at",
                )?;
                for b in bvids {
                    stmt.execute(params![b, now])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    fn record_click(&self, bvid: &str) -> Result<()> {
        self.with(|c| {
            c.prepare_cached("UPDATE exposures SET since_click = 0 WHERE bvid = ?1")?.execute([bvid])?;
            Ok(())
        })
    }

    fn exposures_since_click(&self) -> Result<HashMap<String, u32>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT bvid, since_click FROM exposures WHERE since_click > 0")?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }
}

/// The `kind` column of an event.
pub fn signal_kind(s: &Signal) -> &'static str {
    match s {
        Signal::Impression => "impression",
        Signal::Click => "click",
        Signal::QuickExit => "quick_exit",
        Signal::Completion { .. } => "completion",
        Signal::Rewatch => "rewatch",
        Signal::Like => "like",
        Signal::Coin => "coin",
        Signal::Favorite => "favorite",
        Signal::Follow => "follow",
        Signal::Dislike { .. } => "dislike",
        Signal::Search { .. } => "search",
    }
}

impl EventRepo for SqliteStore {
    fn append_event(&self, ev: &FeedbackEvent) -> Result<()> {
        self.with(|c| insert_event(c, ev))
    }

    fn replace_events(&self, events: &[FeedbackEvent]) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM events", [])?;
            for ev in events {
                insert_event(&tx, ev)?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    fn events_since(&self, at: i64) -> Result<Vec<FeedbackEvent>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT body FROM events WHERE at >= ?1 ORDER BY at, id")?;
            let rows = stmt.query_map([at], |r| r.get::<_, String>(0))?;
            rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
        })
    }

    fn events_of_kinds_since(&self, at: i64, kinds: &[&str]) -> Result<Vec<FeedbackEvent>> {
        let kinds = serde_json::to_string(kinds)?;
        self.with(|c| {
            let mut stmt = c.prepare_cached(
                "SELECT body FROM events WHERE at >= ?1 AND kind IN (SELECT value FROM json_each(?2)) ORDER BY at, id",
            )?;
            let rows = stmt.query_map(params![at, kinds], |r| r.get::<_, String>(0))?;
            rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
        })
    }
}

fn insert_event(c: &Connection, ev: &FeedbackEvent) -> Result<()> {
    let bvid = ev.item.as_ref().map(|i| i.bvid.as_str());
    c.prepare_cached("INSERT INTO events(at, bvid, kind, body) VALUES (?1, ?2, ?3, ?4)")?.execute(params![
        ev.at,
        bvid,
        signal_kind(&ev.signal),
        serde_json::to_string(ev)?
    ])?;
    Ok(())
}

impl HousekeepingRepo for SqliteStore {
    fn prune(&self, now: i64) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM exposures WHERE last_at < ?1", [now - EXPOSURE_KEEP_S])?;
            tx.execute("DELETE FROM watched WHERE at < ?1", [now - WATCHED_KEEP_S])?;
            tx.execute("DELETE FROM events WHERE id <= (SELECT max(id) FROM events) - ?1", [MAX_EVENTS])?;
            tx.commit()?;
            Ok(())
        })
    }
}

impl WatchedRepo for SqliteStore {
    fn mark_watched(&self, bvid: &str, now: i64) -> Result<()> {
        self.with(|c| {
            c.prepare_cached(
                "INSERT INTO watched(bvid, at) VALUES (?1, ?2) ON CONFLICT(bvid) DO UPDATE SET at = excluded.at",
            )?
            .execute(params![bvid, now])?;
            Ok(())
        })
    }

    fn is_watched(&self, bvid: &str) -> Result<bool> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT 1 FROM watched WHERE bvid = ?1")?;
            Ok(stmt.query_row([bvid], |_| Ok(())).optional()?.is_some())
        })
    }

    fn watched_among(&self, bvids: &[&str]) -> Result<HashSet<String>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT 1 FROM watched WHERE bvid = ?1")?;
            let mut out = HashSet::new();
            for &b in bvids {
                if stmt.query_row([b], |_| Ok(())).optional()?.is_some() {
                    out.insert(b.to_owned());
                }
            }
            Ok(out)
        })
    }
}

impl FilterLogRepo for SqliteStore {
    fn append_filter_log(&self, entries: &[FilterLogEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        self.with(|c| {
            let tx = c.transaction()?;
            {
                let mut stmt = tx.prepare_cached(
                    "INSERT INTO filter_log(at, rule_id, category, title, source, reason) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;
                for e in entries {
                    stmt.execute(params![e.at, e.rule_id, e.category, e.title, e.source, e.reason])?;
                }
            }
            // Keep the log bounded.
            tx.execute("DELETE FROM filter_log WHERE id <= (SELECT max(id) FROM filter_log) - ?1", [MAX_FILTER_LOG])?;
            tx.commit()?;
            Ok(())
        })
    }

    fn recent_filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached(
                "SELECT at, rule_id, category, title, source, reason FROM filter_log ORDER BY id DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map([limit as i64], |r| {
                Ok(FilterLogEntry {
                    at: r.get(0)?,
                    rule_id: r.get(1)?,
                    category: r.get(2)?,
                    title: r.get(3)?,
                    source: r.get(4)?,
                    reason: r.get(5)?,
                })
            })?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }
}

impl PlaylistRepo for SqliteStore {
    fn playlists(&self) -> Result<Vec<PlaylistInfo>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached(
                "SELECT p.id, p.name, (SELECT count(*) FROM playlist_items i WHERE i.list_id = p.id)
                 FROM playlists p ORDER BY p.pos, p.id",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok(PlaylistInfo { id: r.get(0)?, name: r.get(1)?, count: r.get::<_, i64>(2)? as usize })
            })?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }

    fn create_playlist(&self, name: &str) -> Result<i64> {
        self.with(|c| {
            c.execute(
                "INSERT INTO playlists(name, pos) VALUES (?1, (SELECT coalesce(max(pos), 0) + 1 FROM playlists))",
                [name],
            )?;
            Ok(c.last_insert_rowid())
        })
    }

    fn rename_playlist(&self, id: i64, name: &str) -> Result<()> {
        self.with(|c| {
            c.execute("UPDATE playlists SET name = ?2 WHERE id = ?1", params![id, name])?;
            Ok(())
        })
    }

    fn delete_playlist(&self, id: i64) -> Result<()> {
        if id == DEFAULT_PLAYLIST {
            return Ok(());
        }
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM playlist_items WHERE list_id = ?1", [id])?;
            tx.execute("DELETE FROM playlists WHERE id = ?1", [id])?;
            tx.commit()?;
            Ok(())
        })
    }

    fn lists_with(&self, bvid: &str) -> Result<Vec<i64>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT list_id FROM playlist_items WHERE bvid = ?1 ORDER BY list_id")?;
            let rows = stmt.query_map([bvid], |r| r.get(0))?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }

    fn playlist(&self, id: i64) -> Result<Vec<VideoCard>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT card FROM playlist_items WHERE list_id = ?1 ORDER BY pos")?;
            let rows = stmt.query_map([id], |r| r.get::<_, String>(0))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(serde_json::from_str(&row?)?);
            }
            Ok(out)
        })
    }

    fn set_playlist(&self, id: i64, items: &[VideoCard]) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM playlist_items WHERE list_id = ?1", [id])?;
            Self::insert_items(&tx, id, items)?;
            tx.commit()?;
            Ok(())
        })
    }

    fn add_to_playlist(&self, id: i64, card: &VideoCard) -> Result<bool> {
        let body = serde_json::to_string(card)?;
        self.with(|c| {
            let added = c
                .prepare_cached(
                    "INSERT OR IGNORE INTO playlist_items(list_id, pos, bvid, card)
                     VALUES (?1, (SELECT coalesce(max(pos), -1) + 1 FROM playlist_items WHERE list_id = ?1), ?2, ?3)",
                )?
                .execute(params![id, card.bvid, body])?;
            Ok(added > 0)
        })
    }

    fn remove_from_playlist(&self, id: i64, bvid: &str) -> Result<()> {
        self.with(|c| {
            c.prepare_cached("DELETE FROM playlist_items WHERE list_id = ?1 AND bvid = ?2")?
                .execute(params![id, bvid])?;
            Ok(())
        })
    }

    fn move_in_playlist(&self, id: i64, from: usize, to: usize) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            let mut order: Vec<String> = {
                let mut stmt = tx.prepare_cached("SELECT bvid FROM playlist_items WHERE list_id = ?1 ORDER BY pos")?;
                let rows = stmt.query_map([id], |r| r.get(0))?;
                rows.collect::<std::result::Result<_, _>>()?
            };
            if from >= order.len() {
                return Ok(());
            }
            let moved = order.remove(from);
            order.insert(to.min(order.len()), moved);
            {
                let mut stmt =
                    tx.prepare_cached("UPDATE playlist_items SET pos = ?3 WHERE list_id = ?1 AND bvid = ?2")?;
                for (pos, bvid) in order.iter().enumerate() {
                    stmt.execute(params![id, bvid, pos as i64])?;
                }
            }
            tx.commit()?;
            Ok(())
        })
    }

    fn replace_playlists(&self, lists: &[(String, Vec<VideoCard>)], default_at: usize) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM playlist_items", [])?;
            tx.execute("DELETE FROM playlists WHERE id != ?1", [DEFAULT_PLAYLIST])?;
            for (i, (name, items)) in lists.iter().enumerate() {
                let id = if i == default_at {
                    DEFAULT_PLAYLIST
                } else {
                    tx.execute(
                        "INSERT INTO playlists(name, pos) VALUES (?1, (SELECT coalesce(max(pos), 0) + 1 FROM playlists))",
                        [name],
                    )?;
                    tx.last_insert_rowid()
                };
                Self::insert_items(&tx, id, items)?;
            }
            tx.commit()?;
            Ok(())
        })
    }
}

fn put_kv(c: &Connection, key: &str, value: &str) -> Result<()> {
    c.prepare_cached(
        "INSERT INTO kv(key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )?
    .execute([key, value])?;
    Ok(())
}

impl KvRepo for SqliteStore {
    fn get_raw(&self, key: &str) -> Result<Option<String>> {
        self.with(|c| {
            Ok(c.prepare_cached("SELECT value FROM kv WHERE key = ?1")?.query_row([key], |r| r.get(0)).optional()?)
        })
    }

    fn put_raw(&self, key: &str, value: &str) -> Result<()> {
        self.with(|c| put_kv(c, key, value))
    }

    fn update_raw(&self, key: &str, f: &mut dyn FnMut(Option<String>) -> Result<String>) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            let old =
                tx.prepare_cached("SELECT value FROM kv WHERE key = ?1")?.query_row([key], |r| r.get(0)).optional()?;
            put_kv(&tx, key, &f(old)?)?;
            tx.commit()?;
            Ok(())
        })
    }
}

fn get_download(c: &Connection, bvid: &str) -> Result<Option<Download>> {
    let body: Option<String> =
        c.prepare_cached("SELECT body FROM downloads WHERE bvid = ?1")?.query_row([bvid], |r| r.get(0)).optional()?;
    Ok(body.map(|b| serde_json::from_str(&b)).transpose()?)
}

fn put_download_in(c: &Connection, d: &Download) -> Result<()> {
    c.prepare_cached("INSERT OR REPLACE INTO downloads(bvid, created_at, body) VALUES (?1, ?2, ?3)")?
        .execute(params![d.card.bvid, d.created_at, serde_json::to_string(d)?])?;
    Ok(())
}

impl DownloadRepo for SqliteStore {
    fn downloads(&self) -> Result<Vec<Download>> {
        self.with(|c| {
            let mut stmt = c.prepare_cached("SELECT body FROM downloads ORDER BY created_at DESC")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(serde_json::from_str(&row?)?);
            }
            Ok(out)
        })
    }

    fn download(&self, bvid: &str) -> Result<Option<Download>> {
        self.with(|c| get_download(c, bvid))
    }

    fn put_download(&self, d: &Download) -> Result<()> {
        self.with(|c| put_download_in(c, d))
    }

    fn update_download(&self, bvid: &str, f: &mut dyn FnMut(&mut Download)) -> Result<bool> {
        self.with(|c| {
            let tx = c.transaction()?;
            let Some(mut d) = get_download(&tx, bvid)? else { return Ok(false) };
            f(&mut d);
            put_download_in(&tx, &d)?;
            tx.commit()?;
            Ok(true)
        })
    }

    fn remove_download(&self, bvid: &str) -> Result<()> {
        self.with(|c| {
            c.prepare_cached("DELETE FROM downloads WHERE bvid = ?1")?.execute(params![bvid])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubi_model::{ItemRef, SourceKind};

    fn card(bvid: &str, source: SourceKind) -> VideoCard {
        VideoCard { bvid: bvid.into(), sources: vec![source], ..Default::default() }
    }

    fn download(bvid: &str, at: i64) -> Download {
        use rubi_model::DownloadState;
        Download {
            card: card(bvid, SourceKind::Popular),
            quality: 80,
            state: DownloadState::Queued,
            done_bytes: 0,
            total_bytes: 0,
            created_at: at,
        }
    }

    #[test]
    fn downloads_are_listed_newest_first_and_replaced_in_place() {
        use rubi_model::DownloadState;
        let s = SqliteStore::in_memory().unwrap();
        s.put_download(&download("a", 1)).unwrap();
        s.put_download(&download("b", 2)).unwrap();
        s.put_download(&Download { state: DownloadState::Done, ..download("a", 1) }).unwrap();
        let all = s.downloads().unwrap();
        assert_eq!(all.iter().map(|d| d.card.bvid.as_str()).collect::<Vec<_>>(), ["b", "a"]);
        assert_eq!(all[1].state, DownloadState::Done);
        assert!(s.update_download("b", &mut |d| d.done_bytes = 7).unwrap());
        assert_eq!(s.download("b").unwrap().unwrap().done_bytes, 7);
        assert!(!s.update_download("zz", &mut |d| d.done_bytes = 7).unwrap());
        s.remove_download("b").unwrap();
        assert_eq!(s.downloads().unwrap().len(), 1);
    }

    #[test]
    fn candidates_merge_sources_and_expire() {
        let s = SqliteStore::in_memory().unwrap();
        s.put_candidates(&[(card("a", SourceKind::Popular), 1.0)], 100, 0).unwrap();
        s.put_candidates(&[(card("a", SourceKind::Ranking), 0.85)], 100, 10).unwrap();
        let fresh = s.fresh_candidates(50).unwrap();
        assert_eq!(fresh.len(), 1);
        assert_eq!(fresh[0].0.sources, vec![SourceKind::Ranking, SourceKind::Popular]);
        assert!((fresh[0].1 - 0.85).abs() < 1e-6);
        assert!(s.fresh_candidates(200).unwrap().is_empty());
        assert_eq!(s.purge_expired(200).unwrap(), 1);
    }

    #[test]
    fn exposures_reset_on_click() {
        let s = SqliteStore::in_memory().unwrap();
        let ids = vec!["a".to_string(), "b".to_string()];
        s.record_impressions(&ids, 1).unwrap();
        s.record_impressions(&ids, 2).unwrap();
        s.record_click("a").unwrap();
        let e = s.exposures_since_click().unwrap();
        assert_eq!(e.get("a"), None);
        assert_eq!(e.get("b"), Some(&2));
    }

    #[test]
    fn events_watched_log_and_kv_round_trip() {
        let s = SqliteStore::in_memory().unwrap();
        let ev = FeedbackEvent {
            signal: Signal::Like,
            item: Some(ItemRef { bvid: "a".into(), ..Default::default() }),
            at: 5,
        };
        s.append_event(&ev).unwrap();
        s.append_event(&FeedbackEvent { signal: Signal::Click, ..ev.clone() }).unwrap();
        assert_eq!(s.events_since(0).unwrap().len(), 2);
        assert_eq!(s.events_of_kinds_since(0, &["like", "coin"]).unwrap(), vec![ev.clone()]);
        s.replace_events(std::slice::from_ref(&ev)).unwrap();
        assert_eq!(s.events_since(0).unwrap(), vec![ev]);
        s.mark_watched("a", 1).unwrap();
        assert!(s.is_watched("a").unwrap());
        assert!(!s.is_watched("b").unwrap());
        assert_eq!(s.watched_among(&["a", "b"]).unwrap(), HashSet::from(["a".to_owned()]));
        let entry = FilterLogEntry {
            at: 1,
            rule_id: "ad.goto".into(),
            category: "ad".into(),
            title: "t".into(),
            source: "bili_recommend".into(),
            reason: "r".into(),
        };
        s.append_filter_log(std::slice::from_ref(&entry)).unwrap();
        assert_eq!(s.recent_filter_log(10).unwrap(), vec![entry]);
        s.put_json("weights", &vec![1, 2]).unwrap();
        assert_eq!(s.get_json::<Vec<i32>>("weights").unwrap(), Some(vec![1, 2]));
        let after: Vec<i32> = s.update_json("weights", |v: &mut Vec<i32>| v.push(3)).unwrap();
        assert_eq!(after, vec![1, 2, 3]);
        assert_eq!(s.update_json("fresh", |v: &mut Vec<i32>| v.push(1)).unwrap(), vec![1]);
    }

    #[test]
    fn prune_forgets_old_rows() {
        let s = SqliteStore::in_memory().unwrap();
        s.mark_watched("old", 0).unwrap();
        s.mark_watched("new", WATCHED_KEEP_S + 10).unwrap();
        s.record_impressions(&["x".to_owned()], 0).unwrap();
        s.prune(WATCHED_KEEP_S + 10).unwrap();
        assert!(!s.is_watched("old").unwrap());
        assert!(s.is_watched("new").unwrap());
        assert!(s.exposures_since_click().unwrap().is_empty());
    }

    #[test]
    fn playlist_items_change_in_place() {
        let s = SqliteStore::in_memory().unwrap();
        let d = DEFAULT_PLAYLIST;
        assert!(s.add_to_playlist(d, &card("a", SourceKind::Popular)).unwrap());
        assert!(s.add_to_playlist(d, &card("b", SourceKind::Popular)).unwrap());
        assert!(!s.add_to_playlist(d, &card("a", SourceKind::Popular)).unwrap());
        s.add_to_playlist(d, &card("c", SourceKind::Popular)).unwrap();
        s.move_in_playlist(d, 2, 0).unwrap();
        s.remove_from_playlist(d, "a").unwrap();
        let ids: Vec<String> = s.playlist(d).unwrap().into_iter().map(|c| c.bvid).collect();
        assert_eq!(ids, ["c", "b"]);
        s.replace_playlists(&[("x".into(), vec![card("y", SourceKind::Popular)])], 0).unwrap();
        assert_eq!(s.playlists().unwrap().len(), 1);
        assert_eq!(s.playlist(d).unwrap()[0].bvid, "y");
    }
}
