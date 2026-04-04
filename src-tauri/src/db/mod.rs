/// SQLite database layer — see docs/DATABASE.md for full schema.
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use crate::models::build::BuildData;
use crate::models::analysis::AnalysisResult;
use crate::core::oauth::OAuthToken;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| anyhow!("Cannot open DB: {e}"))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        Ok(Database { conn: Mutex::new(conn) })
    }

    pub fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(SCHEMA)?;
        Ok(())
    }

    pub fn save_build(&self, build: &BuildData) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO builds
             (id, name, class_name, ascendancy, level, last_analyzed, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'), datetime('now'))",
            rusqlite::params![
                build.id, build.name, build.class_name,
                build.ascendancy, build.level
            ],
        )?;
        Ok(())
    }

    pub fn load_build(&self, build_id: &str) -> Result<BuildData> {
        // TODO: full build serialization to DB
        // For now, return error — builds are re-parsed from file
        Err(anyhow!("Build {build_id} not in DB — re-import from PoB or OAuth"))
    }

    pub fn save_analysis(&self, analysis: &AnalysisResult) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let json = serde_json::to_string(analysis)?;
        conn.execute(
            "INSERT OR REPLACE INTO analysis_cache (build_id, result_json, created_at)
             VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![analysis.build_id, json],
        )?;
        Ok(())
    }

    pub fn load_analysis(&self, build_id: &str) -> Result<AnalysisResult> {
        let conn = self.conn.lock().unwrap();
        let json: String = conn.query_row(
            "SELECT result_json FROM analysis_cache WHERE build_id = ?1
             ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![build_id],
            |row| row.get(0),
        ).map_err(|_| anyhow!("No cached analysis for {build_id}"))?;

        serde_json::from_str(&json).map_err(|e| anyhow!("Deserialize failed: {e}"))
    }

    pub fn snapshot_build(&self, build_id: &str, description: &str) -> Result<()> {
        // TODO: serialize + compress build XML for undo history
        Ok(())
    }

    pub fn undo_snapshot(&self, build_id: &str) -> Result<BuildData> {
        Err(anyhow!("Undo not yet implemented"))
    }

    pub fn redo_snapshot(&self, build_id: &str) -> Result<BuildData> {
        Err(anyhow!("Redo not yet implemented"))
    }

    pub fn save_oauth_token(&self, token: &OAuthToken) -> Result<()> {
        // TODO: encrypt with AES-256 before storing
        let conn = self.conn.lock().unwrap();
        let json = serde_json::to_string(token)?;
        conn.execute(
            "INSERT OR REPLACE INTO oauth_tokens (provider, token_json, created_at)
             VALUES ('poe', ?1, datetime('now'))",
            rusqlite::params![json],
        )?;
        Ok(())
    }

    pub fn load_oauth_token(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let json: String = conn.query_row(
            "SELECT token_json FROM oauth_tokens WHERE provider = 'poe'
             ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).map_err(|_| anyhow!("No OAuth token stored — connect your PoE account"))?;

        let token: OAuthToken = serde_json::from_str(&json)?;
        Ok(token.access_token)
    }
}

// Full schema — see docs/DATABASE.md
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS builds (
    id           TEXT PRIMARY KEY,
    name         TEXT,
    class_name   TEXT NOT NULL DEFAULT '',
    ascendancy   TEXT NOT NULL DEFAULT '',
    level        INTEGER NOT NULL DEFAULT 1,
    main_skill   TEXT,
    archetype    TEXT,
    total_dps    REAL,
    total_life   INTEGER,
    score        INTEGER,
    last_analyzed TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS analysis_cache (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    build_id    TEXT NOT NULL REFERENCES builds(id),
    result_json TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS build_snapshots (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    build_id    TEXT NOT NULL REFERENCES builds(id),
    xml_content BLOB NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_snapshots_build ON build_snapshots(build_id, created_at DESC);

CREATE TABLE IF NOT EXISTS oauth_tokens (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    provider    TEXT NOT NULL,
    token_json  TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS price_cache (
    item_name   TEXT PRIMARY KEY,
    price_chaos REAL NOT NULL,
    price_div   REAL NOT NULL,
    listings    INTEGER,
    fetched_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS alerts (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    item_name       TEXT NOT NULL,
    threshold_div   REAL NOT NULL,
    comparison      TEXT NOT NULL,
    notify_method   TEXT NOT NULL,
    active          INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS map_runs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    zone_name       TEXT NOT NULL,
    zone_type       TEXT,
    started_at      TEXT NOT NULL,
    ended_at        TEXT,
    duration_secs   INTEGER,
    items_found     INTEGER DEFAULT 0,
    currency_found  REAL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS wealth_snapshots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    total_div       REAL NOT NULL,
    breakdown_json  TEXT,
    snapshotted_at  TEXT NOT NULL
);
"#;
