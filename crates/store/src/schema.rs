use rusqlite::Connection;

/// Each entry upgrades the schema by one version; never edit a shipped entry.
const MIGRATIONS: &[&str] = &["CREATE TABLE candidates(
        bvid TEXT PRIMARY KEY,
        card TEXT NOT NULL,
        penalty REAL NOT NULL,
        fetched_at INTEGER NOT NULL,
        expires_at INTEGER NOT NULL
     );
     CREATE INDEX candidates_expiry ON candidates(expires_at);
     CREATE TABLE exposures(
        bvid TEXT PRIMARY KEY,
        since_click INTEGER NOT NULL,
        total INTEGER NOT NULL,
        last_at INTEGER NOT NULL
     );
     CREATE TABLE events(
        id INTEGER PRIMARY KEY,
        at INTEGER NOT NULL,
        bvid TEXT,
        kind TEXT NOT NULL,
        body TEXT NOT NULL
     );
     CREATE INDEX events_at ON events(at);
     CREATE TABLE watched(bvid TEXT PRIMARY KEY, at INTEGER NOT NULL);
     CREATE TABLE filter_log(
        id INTEGER PRIMARY KEY,
        at INTEGER NOT NULL,
        rule_id TEXT NOT NULL,
        category TEXT NOT NULL,
        title TEXT NOT NULL,
        source TEXT NOT NULL,
        reason TEXT NOT NULL
     );
     CREATE TABLE kv(key TEXT PRIMARY KEY, value TEXT NOT NULL);"];

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let current: usize = conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))? as usize;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        conn.execute_batch(&format!("BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;", i + 1))?;
    }
    Ok(())
}
