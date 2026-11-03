use rusqlite::Connection;

/// Each entry upgrades the schema by one version; never edit a shipped entry.
const MIGRATIONS: &[&str] = &[
    "CREATE TABLE candidates(
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
     CREATE TABLE kv(key TEXT PRIMARY KEY, value TEXT NOT NULL);",
    "CREATE TABLE playlist(
        pos INTEGER NOT NULL,
        bvid TEXT PRIMARY KEY,
        card TEXT NOT NULL
     );",
    "CREATE TABLE downloads(
        bvid TEXT PRIMARY KEY,
        created_at INTEGER NOT NULL,
        body TEXT NOT NULL
     );",
    // Several named play lists; the old single list becomes 默认 (id 1).
    "CREATE TABLE playlists(
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        pos INTEGER NOT NULL
     );
     INSERT INTO playlists(id, name, pos) VALUES (1, '默认', 0);
     CREATE TABLE playlist_items(
        list_id INTEGER NOT NULL,
        pos INTEGER NOT NULL,
        bvid TEXT NOT NULL,
        card TEXT NOT NULL,
        PRIMARY KEY(list_id, bvid)
     );
     INSERT INTO playlist_items(list_id, pos, bvid, card) SELECT 1, pos, bvid, card FROM playlist;
     DROP TABLE playlist;",
    // Pruning by age, and the events read by kind (see HousekeepingRepo, EventRepo).
    "CREATE INDEX exposures_last ON exposures(last_at);
     CREATE INDEX watched_at ON watched(at);
     CREATE INDEX events_kind_at ON events(kind, at);",
];

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let current: usize = conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))? as usize;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        conn.execute_batch(&format!("BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;", i + 1))?;
    }
    Ok(())
}
