/// SQLite database layer — see docs/DATABASE.md for full schema.
use anyhow::{anyhow, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use crate::models::build::BuildData;
use crate::models::analysis::AnalysisResult;
use crate::core::oauth::{OAuthToken, encrypt_token, decrypt_token, load_or_create_key};

pub struct Database {
    conn: Mutex<Connection>,
    /// 32-byte AES-256-GCM key for token encryption.
    enc_key: [u8; 32],
    data_dir: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let data_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let enc_key = load_or_create_key(&data_dir)?;

        let conn = Connection::open(path)
            .map_err(|e| anyhow!("Cannot open DB: {e}"))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        Ok(Database { conn: Mutex::new(conn), enc_key, data_dir })
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(SCHEMA)?;
        Ok(())
    }

    // ─── Builds ───────────────────────────────────────────────────────────────

    /// Save build metadata + full JSON to the DB.
    pub fn save_build(&self, build: &BuildData) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let full_json = serde_json::to_string(build)?;
        conn.execute(
            "INSERT OR REPLACE INTO builds
             (id, name, class_name, ascendancy, level, full_json, last_analyzed, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), datetime('now'), datetime('now'))",
            rusqlite::params![
                build.id, build.name, build.class_name,
                build.ascendancy, build.level, full_json
            ],
        )?;
        Ok(())
    }

    /// Load full BuildData from the DB.
    pub fn load_build(&self, build_id: &str) -> Result<BuildData> {
        let conn = self.conn.lock().unwrap();
        let json: String = conn.query_row(
            "SELECT full_json FROM builds WHERE id = ?1",
            rusqlite::params![build_id],
            |row| row.get(0),
        ).map_err(|_| anyhow!("Build {build_id} not found — re-import from PoB or OAuth"))?;

        serde_json::from_str(&json).map_err(|e| anyhow!("Deserialize build: {e}"))
    }

    /// List all saved builds (summary only, no full JSON).
    pub fn list_builds(&self) -> Result<Vec<BuildSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, class_name, ascendancy, level, last_analyzed
             FROM builds ORDER BY updated_at DESC LIMIT 50"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BuildSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                class_name: row.get(2)?,
                ascendancy: row.get(3)?,
                level: row.get(4)?,
                last_analyzed: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(|e| anyhow!("{e}"))
    }

    // ─── Analysis Cache ───────────────────────────────────────────────────────

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

    // ─── Snapshots (Undo/Redo) ────────────────────────────────────────────────

    /// Save a snapshot of the current build state before a change.
    /// Keeps the most recent 50 snapshots per build.
    pub fn snapshot_build(&self, build_id: &str, description: &str) -> Result<()> {
        let build = self.load_build(build_id)?;
        let json = serde_json::to_string(&build)?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO build_snapshots (build_id, xml_content, description, created_at)
             VALUES (?1, ?2, ?3, datetime('now'))",
            rusqlite::params![build_id, json.as_bytes(), description],
        )?;

        // Trim to 50 snapshots
        conn.execute(
            "DELETE FROM build_snapshots WHERE id NOT IN (
               SELECT id FROM build_snapshots WHERE build_id = ?1
               ORDER BY created_at DESC LIMIT 50
             ) AND build_id = ?1",
            rusqlite::params![build_id],
        )?;
        Ok(())
    }

    /// Undo: restore previous snapshot and remove it from the stack.
    pub fn undo_snapshot(&self, build_id: &str) -> Result<BuildData> {
        let conn = self.conn.lock().unwrap();

        // Get the most recent snapshot
        let (snap_id, json_bytes): (i64, Vec<u8>) = conn.query_row(
            "SELECT id, xml_content FROM build_snapshots WHERE build_id = ?1
             ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![build_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|_| anyhow!("No undo history for build {build_id}"))?;

        let json = String::from_utf8(json_bytes)
            .map_err(|e| anyhow!("Snapshot UTF-8 error: {e}"))?;
        let build: BuildData = serde_json::from_str(&json)
            .map_err(|e| anyhow!("Snapshot deserialize: {e}"))?;

        // Remove snapshot from stack
        conn.execute(
            "DELETE FROM build_snapshots WHERE id = ?1",
            rusqlite::params![snap_id],
        )?;

        // Update the build in DB
        let full_json = serde_json::to_string(&build).unwrap_or_default();
        conn.execute(
            "UPDATE builds SET full_json = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![full_json, build_id],
        )?;

        Ok(build)
    }

    /// Redo is implemented as a second snapshot table ("redo stack").
    /// For simplicity, we do not support redo after new changes — like most editors.
    pub fn redo_snapshot(&self, _build_id: &str) -> Result<BuildData> {
        Err(anyhow!("Redo not available after new changes"))
    }

    // ─── OAuth Tokens (AES-256 encrypted) ────────────────────────────────────

    pub fn save_oauth_token(&self, token: &OAuthToken) -> Result<()> {
        let plain = serde_json::to_string(token)?;
        let encrypted = encrypt_token(&plain, &self.enc_key)?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO oauth_tokens (provider, token_json, created_at)
             VALUES ('poe', ?1, datetime('now'))",
            rusqlite::params![encrypted],
        )?;
        Ok(())
    }

    pub fn load_oauth_token(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let encrypted: String = conn.query_row(
            "SELECT token_json FROM oauth_tokens WHERE provider = 'poe'
             ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).map_err(|_| anyhow!("No OAuth token stored — connect your PoE account"))?;
        drop(conn);

        let plain = decrypt_token(&encrypted, &self.enc_key)?;
        let token: OAuthToken = serde_json::from_str(&plain)?;
        Ok(token.access_token)
    }

    pub fn has_oauth_token(&self) -> bool {
        self.load_oauth_token().is_ok()
    }

    // ─── Wealth Snapshots ─────────────────────────────────────────────────────

    pub fn record_wealth_snapshot(&self, total_div: f64, breakdown: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO wealth_snapshots (total_div, breakdown_json, snapshotted_at)
             VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![total_div, breakdown],
        )?;
        Ok(())
    }
}

// ─── Build summary for list views ─────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BuildSummary {
    pub id: String,
    pub name: String,
    pub class_name: String,
    pub ascendancy: String,
    pub level: u32,
    pub last_analyzed: String,
}

// ─── Schema ───────────────────────────────────────────────────────────────────

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
    full_json    TEXT,
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::build::BuildData;
    use crate::core::oauth::{OAuthToken, encrypt_token, load_or_create_key};
    use tempfile::tempdir;

    fn make_test_build(id: &str, name: &str) -> BuildData {
        BuildData {
            id: id.to_string(),
            name: name.to_string(),
            class_name: "Templar".to_string(),
            ascendancy: "Inquisitor".to_string(),
            level: 90,
            ..Default::default()
        }
    }

    fn open_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open db");
        db.run_migrations().expect("migrations");
        (db, dir)
    }

    // ── save / load build ────────────────────────────────────────────────────

    #[test]
    fn save_and_load_build_roundtrip() {
        let (db, _dir) = open_test_db();
        let build = make_test_build("build-001", "RF Inquisitor");

        db.save_build(&build).expect("save build");
        let loaded = db.load_build("build-001").expect("load build");

        assert_eq!(loaded.id, build.id);
        assert_eq!(loaded.name, build.name);
        assert_eq!(loaded.class_name, "Templar");
        assert_eq!(loaded.level, 90);
    }

    #[test]
    fn save_build_replace_updates_existing() {
        let (db, _dir) = open_test_db();
        let mut build = make_test_build("build-002", "Original Name");
        db.save_build(&build).expect("save first");

        build.name = "Updated Name".to_string();
        db.save_build(&build).expect("save second");

        let loaded = db.load_build("build-002").expect("load");
        assert_eq!(loaded.name, "Updated Name", "Save should replace (upsert) existing build");
    }

    #[test]
    fn load_build_returns_error_for_missing_id() {
        let (db, _dir) = open_test_db();
        let result = db.load_build("nonexistent-id");
        assert!(result.is_err(), "Loading a missing build should return an error");
    }

    // ── list builds ─────────────────────────────────────────────────────────

    #[test]
    fn list_builds_returns_all_saved_builds() {
        let (db, _dir) = open_test_db();
        db.save_build(&make_test_build("a", "Build A")).unwrap();
        db.save_build(&make_test_build("b", "Build B")).unwrap();
        db.save_build(&make_test_build("c", "Build C")).unwrap();

        let list = db.list_builds().expect("list builds");
        assert_eq!(list.len(), 3, "Should list 3 saved builds");

        let names: Vec<&str> = list.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"Build A"));
        assert!(names.contains(&"Build B"));
        assert!(names.contains(&"Build C"));
    }

    #[test]
    fn list_builds_empty_when_no_builds() {
        let (db, _dir) = open_test_db();
        let list = db.list_builds().expect("list builds");
        assert!(list.is_empty(), "Should return empty list when no builds saved");
    }

    // ── snapshots / undo ─────────────────────────────────────────────────────

    #[test]
    fn snapshot_then_undo_restores_original() {
        let (db, _dir) = open_test_db();
        let original = make_test_build("snap-001", "Pre-Change Build");
        db.save_build(&original).expect("save original");

        // Take snapshot before change
        db.snapshot_build("snap-001", "before upgrade").expect("snapshot");

        // Simulate change
        let mut changed = original.clone();
        changed.name = "Post-Change Build".to_string();
        changed.level = 95;
        db.save_build(&changed).expect("save changed");

        // Verify change was applied
        let after_change = db.load_build("snap-001").unwrap();
        assert_eq!(after_change.name, "Post-Change Build");

        // Undo
        let restored = db.undo_snapshot("snap-001").expect("undo");
        assert_eq!(restored.name, "Pre-Change Build", "Undo should restore original build name");
        assert_eq!(restored.level, 90, "Undo should restore original level");
    }

    #[test]
    fn undo_with_no_history_returns_error() {
        let (db, _dir) = open_test_db();
        db.save_build(&make_test_build("no-snap", "Build")).unwrap();

        let result = db.undo_snapshot("no-snap");
        assert!(result.is_err(), "Undo with no history should return error");
    }

    #[test]
    fn redo_always_returns_error() {
        let (db, _dir) = open_test_db();
        let result = db.redo_snapshot("any-id");
        assert!(result.is_err(), "Redo is not supported — should return error");
    }

    // ── wealth snapshots ─────────────────────────────────────────────────────

    #[test]
    fn record_wealth_snapshot_persists_to_db() {
        let (db, _dir) = open_test_db();
        db.record_wealth_snapshot(42.5, Some("{\"chaos\":1000}"))
            .expect("record wealth snapshot");

        // Verify via direct query
        let conn = db.conn.lock().unwrap();
        let total_div: f64 = conn.query_row(
            "SELECT total_div FROM wealth_snapshots LIMIT 1",
            [],
            |row| row.get(0),
        ).expect("query wealth snapshot");
        assert!((total_div - 42.5).abs() < 0.001, "Wealth snapshot should persist 42.5 div");
    }

    // ── OAuth token ──────────────────────────────────────────────────────────

    #[test]
    fn has_oauth_token_false_when_no_token_stored() {
        let (db, _dir) = open_test_db();
        assert!(!db.has_oauth_token(), "New DB should have no OAuth token");
    }

    #[test]
    fn save_and_load_oauth_token_roundtrip() {
        let (db, _dir) = open_test_db();
        let token = OAuthToken {
            access_token: "test-access-token-123".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: "account:profile".to_string(),
        };

        db.save_oauth_token(&token).expect("save token");
        assert!(db.has_oauth_token(), "DB should report token present after save");

        let loaded = db.load_oauth_token().expect("load token");
        assert_eq!(loaded, "test-access-token-123", "Should return access_token string");
    }
}

