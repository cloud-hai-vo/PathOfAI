/// Settings commands — configure AI providers, league, app preferences.
use tauri::State;
use crate::AppState;
use crate::core::cloud_ai::{CloudProvider, ConnectionTestResult};
use serde::{Deserialize, Serialize};

// ─── AppSettings model ────────────────────────────────────────────────────────

/// Persistent user-facing app settings saved to `config/settings.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Active PoE league name (e.g. "Settlers of Kalguur").
    pub league:              String,
    /// Target boss for damage calculations.
    pub default_boss:        String,
    /// poe.ninja price refresh interval in seconds.
    pub price_refresh_secs:  u32,
    /// Currency for price display: "divine" or "chaos".
    pub price_currency:      String,
    /// Notification sound enabled.
    pub sound_enabled:       bool,
    /// Custom PoB watch directory (empty = auto-detect).
    pub pob_watch_dir:       String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            league:             "".to_string(),
            default_boss:       "Pinnacle".to_string(),
            price_refresh_secs: 300,
            price_currency:     "divine".to_string(),
            sound_enabled:      true,
            pob_watch_dir:      "".to_string(),
        }
    }
}

/// Test an AI provider API key before saving it.
#[tauri::command]
pub async fn test_cloud_ai(
    provider: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<ConnectionTestResult, String> {
    let provider = parse_provider(&provider)?;
    Ok(crate::core::cloud_ai::test_connection(&provider, &api_key, &state.http).await)
}

/// Save an API key for a cloud AI provider.
#[tauri::command]
pub async fn save_ai_key(
    provider: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let provider = parse_provider(&provider)?;
    state.ai_keys.lock().unwrap().set(provider, api_key);
    Ok(())
}

/// Remove a saved API key.
#[tauri::command]
pub async fn remove_ai_key(
    provider: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let provider = parse_provider(&provider)?;
    state.ai_keys.lock().unwrap().remove(&provider);
    Ok(())
}

/// List which providers currently have a key configured.
#[tauri::command]
pub async fn get_configured_providers(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let keys = state.ai_keys.lock().unwrap();
    Ok(keys.configured_providers().iter().map(|p| p.name().to_string()).collect())
}

/// Return the directory currently being watched for PoB file changes.
#[tauri::command]
pub async fn get_pob_watch_dir(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let dir = state.db.data_dir().join("builds");
    let path = std::env::var("POE_POB_DIR")
        .unwrap_or_else(|_| dir.to_string_lossy().to_string());
    Ok(path)
}

/// Save user-facing app settings to `config/settings.json`.
#[tauri::command]
pub async fn save_settings(
    settings_json: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings: AppSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("invalid settings JSON: {e}"))?;
    let path = state.db.data_dir().join("config").join("settings.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    crate::core::session_persistence::atomic_write(&path, &data)
}

/// Load current app settings (returns defaults if file is missing).
#[tauri::command]
pub async fn load_settings(
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let path = state.db.data_dir().join("config").join("settings.json");
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
        Err(_)    => Ok(AppSettings::default()),
    }
}

/// Change the PoB watch directory at runtime.
/// Returns the resolved path actually being watched.
#[tauri::command]
pub async fn watch_pob_directory(
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Persist the new path into settings so it survives restarts
    let settings_path = state.db.data_dir().join("config").join("settings.json");
    let mut settings: AppSettings = match std::fs::read(&settings_path) {
        Ok(b) => serde_json::from_slice(&b).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    };
    settings.pob_watch_dir = path.clone();
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    crate::core::session_persistence::atomic_write(&settings_path, &data)?;
    Ok(path)
}

fn parse_provider(s: &str) -> Result<CloudProvider, String> {
    match s.to_lowercase().as_str() {
        "claude"     => Ok(CloudProvider::Claude),
        "gpt4" | "gpt-4" | "openai" => Ok(CloudProvider::Gpt4),
        "gemini"     => Ok(CloudProvider::Gemini),
        "ollama"     => Ok(CloudProvider::Ollama),
        "openrouter" => Ok(CloudProvider::OpenRouter),
        other => Err(format!("Unknown provider: {other}")),
    }
}
