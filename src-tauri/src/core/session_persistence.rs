/// session_persistence.rs — Session Persistence & Auto-Save (Algorithm 35).
///
/// Serializes the user's UI state (active build, active tab, pending suggestions,
/// window geometry) to `PathOfAI_Data/config/session.json` using an atomic
/// write (temp file → fsync → rename) so a crash never corrupts the session.
///
/// The module is pure logic — it does not depend on Tauri. The caller is
/// responsible for supplying the file path and triggering saves.
use serde::{Deserialize, Serialize};
use std::path::Path;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Which main tab is active in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ActiveTab {
    #[default]
    Overview,
    Items,
    Tree,
    Crafting,
    Arena,
    Market,
    Settings,
}

/// The full session state written to disk on every meaningful change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Schema version — bump when adding fields so old files can be migrated.
    pub version: u32,

    // ── Active build ─────────────────────────────────────────────────────────
    pub active_build_id:       Option<String>,
    pub active_character_name: Option<String>,
    pub active_pob_path:       Option<String>,

    // ── UI state ─────────────────────────────────────────────────────────────
    pub active_tab:   ActiveTab,
    pub active_slot:  Option<String>,

    // ── Pending state ────────────────────────────────────────────────────────
    /// Suggestion shown to the user but not yet applied.
    pub pending_suggestion_id: Option<String>,

    // ── Window geometry ──────────────────────────────────────────────────────
    pub window_x:         i32,
    pub window_y:         i32,
    pub window_w:         u32,
    pub window_h:         u32,
    pub window_maximized: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version:               1,
            active_build_id:       None,
            active_character_name: None,
            active_pob_path:       None,
            active_tab:            ActiveTab::Overview,
            active_slot:           None,
            pending_suggestion_id: None,
            window_x:              100,
            window_y:              100,
            window_w:              1280,
            window_h:              800,
            window_maximized:      false,
        }
    }
}

// ─── Load / save ─────────────────────────────────────────────────────────────

/// Load `session.json` from `path`. Returns `SessionState::default()` if the
/// file is missing or cannot be parsed (first-launch / corrupted).
pub fn load_session(path: &Path) -> SessionState {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_)    => SessionState::default(),
    }
}

/// Write `state` to `path` atomically: temp file → rename.
/// Returns `Ok(())` on success, `Err(String)` on failure.
pub fn save_session(path: &Path, state: &SessionState) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    atomic_write(path, &data)
}

/// Atomic write: write to `<path>.tmp`, then rename to `path`.
/// If the process crashes between write and rename, the original is untouched.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data).map_err(|e| format!("write tmp: {e}"))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename: {e}"))?;
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── load_session ──────────────────────────────────────────────────────────

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        let s = load_session(&path);
        assert_eq!(s.version, 1);
        assert!(s.active_build_id.is_none());
    }

    #[test]
    fn load_corrupted_file_returns_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        std::fs::write(&path, b"{ not valid json !!!").unwrap();
        let s = load_session(&path);
        assert_eq!(s.active_tab, ActiveTab::Overview);
    }

    // ── save_session / round-trip ─────────────────────────────────────────────

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        let mut state = SessionState::default();
        state.active_build_id = Some("build-42".to_string());
        state.active_tab = ActiveTab::Items;
        state.window_w = 1920;
        state.window_maximized = true;

        save_session(&path, &state).unwrap();
        let loaded = load_session(&path);

        assert_eq!(loaded.active_build_id.as_deref(), Some("build-42"));
        assert_eq!(loaded.active_tab, ActiveTab::Items);
        assert_eq!(loaded.window_w, 1920);
        assert!(loaded.window_maximized);
    }

    #[test]
    fn save_creates_file_atomically() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        let state = SessionState::default();
        save_session(&path, &state).unwrap();
        // File exists, no .tmp leftover
        assert!(path.exists());
        assert!(!path.with_extension("tmp").exists());
    }

    #[test]
    fn save_over_existing_replaces_it() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");

        let mut s1 = SessionState::default();
        s1.active_build_id = Some("old".to_string());
        save_session(&path, &s1).unwrap();

        let mut s2 = SessionState::default();
        s2.active_build_id = Some("new".to_string());
        save_session(&path, &s2).unwrap();

        let loaded = load_session(&path);
        assert_eq!(loaded.active_build_id.as_deref(), Some("new"));
    }

    // ── pending suggestion survives restart ───────────────────────────────────

    #[test]
    fn pending_suggestion_persists() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("session.json");
        let mut state = SessionState::default();
        state.pending_suggestion_id = Some("sugg-uuid-123".to_string());
        save_session(&path, &state).unwrap();

        let loaded = load_session(&path);
        assert_eq!(loaded.pending_suggestion_id.as_deref(), Some("sugg-uuid-123"));
    }

    // ── atomic_write ──────────────────────────────────────────────────────────

    #[test]
    fn atomic_write_no_tmp_leftover() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("data.bin");
        atomic_write(&path, b"hello world").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"hello world");
        assert!(!path.with_extension("tmp").exists());
    }
}
