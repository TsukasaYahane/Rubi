//! Local persistence. Callers depend on the repository traits; [`SqliteStore`]
//! implements all of them over a single SQLite file.

mod schema;

use std::collections::HashMap;
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
}

pub trait WatchedRepo {
    fn mark_watched(&self, bvid: &str, now: i64) -> Result<()>;
    fn is_watched(&self, bvid: &str) -> Result<bool>;
}

pub trait FilterLogRepo {
    fn append_filter_log(&self, entries: &[FilterLogEntry]) -> Result<()>;
    fn recent_filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>>;
}

/// The custom play list, in order.
pub trait PlaylistRepo {
    fn playlist(&self) -> Result<Vec<VideoCard>>;
    /// Replaces the whole list (it is short).
    fn set_playlist(&self, items: &[VideoCard]) -> Result<()>;
}

/// 离线缓存 records.
pub trait DownloadRepo {
    /// Newest first.
    fn downloads(&self) -> Result<Vec<Download>>;
    /// Adds or replaces the record of `d.card.bvid`.
    fn put_download(&self, d: &Download) -> Result<()>;
    fn remove_download(&self, bvid: &str) -> Result<()>;
}

/// Typed key/value settings (rule table, weights, profile…).
pub trait KvRepo {
    fn get_raw(&self, key: &str) -> Result<Option<String>>;
    fn put_raw(&self, key: &str, value: &str) -> Result<()>;

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
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        schema::migrate(&conn)?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn with<T>(&self, f: impl FnOnce(&mut Connection) -> Result<T>) -> Result<T> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut conn)
    }
}

impl CandidateRepo for SqliteStore {
    fn put_candidates(&self, cards: &[(VideoCard, f32)], ttl_s: i64, now: i64) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO candidates(bvid, card, penalty, fetched_at, expires_at) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(bvid) DO UPDATE SET card = excluded.card, penalty = min(penalty, excluded.penalty),
                       fetched_at = excluded.fetched_at, expires_at = excluded.expires_at",
                )?;
                for (card, penalty) in cards {
                    let mut merged = card.clone();
                    // Keep sources from earlier fetches so the topic factor sees every source.
                    if let Some(old) = tx
                        .query_row("SELECT card FROM candidates WHERE bvid = ?1", [&card.bvid], |r| {
                            r.get::<_, String>(0)
                        })
                        .optional()?
                    {
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
            let mut stmt = c.prepare("SELECT card, penalty FROM candidates WHERE expires_at > ?1")?;
            let rows = stmt.query_map([now], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f32>(1)?)))?;
            rows.map(|r| {
                let (json, p) = r?;
                Ok((serde_json::from_str(&json)?, p))
            })
            .collect()
        })
    }

    fn purge_expired(&self, now: i64) -> Result<usize> {
        self.with(|c| Ok(c.execute("DELETE FROM candidates WHERE expires_at <= ?1", [now])?))
    }
}

impl ExposureRepo for SqliteStore {
    fn record_impressions(&self, bvids: &[String], now: i64) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            {
                let mut stmt = tx.prepare(
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
        self.with(|c| Ok(c.execute("UPDATE exposures SET since_click = 0 WHERE bvid = ?1", [bvid]).map(|_| ())?))
    }

    fn exposures_since_click(&self) -> Result<HashMap<String, u32>> {
        self.with(|c| {
            let mut stmt = c.prepare("SELECT bvid, since_click FROM exposures WHERE since_click > 0")?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)))?;
            Ok(rows.collect::<std::result::Result<_, _>>()?)
        })
    }
}

fn signal_kind(s: &Signal) -> &'static str {
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
        let bvid = ev.item.as_ref().map(|i| i.bvid.clone());
        let body = serde_json::to_string(ev)?;
        self.with(|c| {
            c.execute(
                "INSERT INTO events(at, bvid, kind, body) VALUES (?1, ?2, ?3, ?4)",
                params![ev.at, bvid, signal_kind(&ev.signal), body],
            )?;
            Ok(())
        })
    }

    fn events_since(&self, at: i64) -> Result<Vec<FeedbackEvent>> {
        self.with(|c| {
            let mut stmt = c.prepare("SELECT body FROM events WHERE at >= ?1 ORDER BY at, id")?;
            let rows = stmt.query_map([at], |r| r.get::<_, String>(0))?;
            rows.map(|r| Ok(serde_json::from_str(&r?)?)).collect()
        })
    }
}

impl WatchedRepo for SqliteStore {
    fn mark_watched(&self, bvid: &str, now: i64) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT INTO watched(bvid, at) VALUES (?1, ?2) ON CONFLICT(bvid) DO UPDATE SET at = excluded.at",
                params![bvid, now],
            )?;
            Ok(())
        })
    }

    fn is_watched(&self, bvid: &str) -> Result<bool> {
        self.with(|c| {
            Ok(c.query_row("SELECT 1 FROM watched WHERE bvid = ?1", [bvid], |_| Ok(())).optional()?.is_some())
        })
    }
}

impl FilterLogRepo for SqliteStore {
    fn append_filter_log(&self, entries: &[FilterLogEntry]) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            {
                let mut stmt = tx.prepare(
                    "INSERT INTO filter_log(at, rule_id, category, title, source, reason) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;
                for e in entries {
                    stmt.execute(params![e.at, e.rule_id, e.category, e.title, e.source, e.reason])?;
                }
            }
            // Keep the log bounded.
            tx.execute("DELETE FROM filter_log WHERE id <= (SELECT max(id) FROM filter_log) - 5000", [])?;
            tx.commit()?;
            Ok(())
        })
    }

    fn recent_filter_log(&self, limit: usize) -> Result<Vec<FilterLogEntry>> {
        self.with(|c| {
            let mut stmt = c.prepare(
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
    fn playlist(&self) -> Result<Vec<VideoCard>> {
        self.with(|c| {
            let mut stmt = c.prepare("SELECT card FROM playlist ORDER BY pos")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(serde_json::from_str(&row?)?);
            }
            Ok(out)
        })
    }

    fn set_playlist(&self, items: &[VideoCard]) -> Result<()> {
        self.with(|c| {
            let tx = c.transaction()?;
            tx.execute("DELETE FROM playlist", [])?;
            for (pos, card) in items.iter().enumerate() {
                tx.execute(
                    "INSERT OR IGNORE INTO playlist(pos, bvid, card) VALUES (?1, ?2, ?3)",
                    params![pos as i64, card.bvid, serde_json::to_string(card)?],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }
}

impl KvRepo for SqliteStore {
    fn get_raw(&self, key: &str) -> Result<Option<String>> {
        self.with(|c| Ok(c.query_row("SELECT value FROM kv WHERE key = ?1", [key], |r| r.get(0)).optional()?))
    }

    fn put_raw(&self, key: &str, value: &str) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT INTO kv(key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [key, value],
            )?;
            Ok(())
        })
    }
}

impl DownloadRepo for SqliteStore {
    fn downloads(&self) -> Result<Vec<Download>> {
        self.with(|c| {
            let mut stmt = c.prepare("SELECT body FROM downloads ORDER BY created_at DESC")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(serde_json::from_str(&row?)?);
            }
            Ok(out)
        })
    }

    fn put_download(&self, d: &Download) -> Result<()> {
        self.with(|c| {
            c.execute(
                "INSERT OR REPLACE INTO downloads(bvid, created_at, body) VALUES (?1, ?2, ?3)",
                params![d.card.bvid, d.created_at, serde_json::to_string(d)?],
            )?;
            Ok(())
        })
    }

    fn remove_download(&self, bvid: &str) -> Result<()> {
        self.with(|c| {
            c.execute("DELETE FROM downloads WHERE bvid = ?1", params![bvid])?;
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

    #[test]
    fn downloads_are_listed_newest_first_and_replaced_in_place() {
        use rubi_model::{Download, DownloadState};
        let s = SqliteStore::in_memory().unwrap();
        let d = |bvid: &str, at| Download {
            card: card(bvid, SourceKind::Popular),
            quality: 80,
            state: DownloadState::Queued,
            done_bytes: 0,
            total_bytes: 0,
            created_at: at,
        };
        s.put_download(&d("a", 1)).unwrap();
        s.put_download(&d("b", 2)).unwrap();
        s.put_download(&Download { state: DownloadState::Done, ..d("a", 1) }).unwrap();
        let all = s.downloads().unwrap();
        assert_eq!(all.iter().map(|d| d.card.bvid.as_str()).collect::<Vec<_>>(), ["b", "a"]);
        assert_eq!(all[1].state, DownloadState::Done);
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
        assert_eq!(s.events_since(0).unwrap(), vec![ev]);
        s.mark_watched("a", 1).unwrap();
        assert!(s.is_watched("a").unwrap());
        assert!(!s.is_watched("b").unwrap());
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
    }
}
