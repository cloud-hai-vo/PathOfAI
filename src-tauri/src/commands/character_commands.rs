use tauri::State;
use crate::AppState;
use crate::models::AnalysisResult;
use crate::core::{characters, oauth};
use crate::db::BuildSummary;
use super::build_commands::analyze_build_data;

/// Load and analyze a character directly from PoE account (OAuth primary path).
#[tauri::command]
pub async fn load_character(
    character_name: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let token = state.db.load_oauth_token()
        .map_err(|_| "Not authenticated — connect your PoE account first".to_string())?;

    let build = characters::fetch_character(&token, &character_name)
        .await
        .map_err(|e| format!("Failed to fetch character: {e}"))?;

    analyze_build_data(build, &state).await
}

/// List all characters on the connected PoE account.
#[tauri::command]
pub async fn list_characters(
    state: State<'_, AppState>,
) -> Result<Vec<characters::CharacterSummary>, String> {
    let token = state.db.load_oauth_token()
        .map_err(|_| "Not authenticated".to_string())?;

    characters::list_characters(&token)
        .await
        .map_err(|e| format!("Failed to list characters: {e}"))
}

/// Switch active character and re-analyze.
#[tauri::command]
pub async fn switch_character(
    character_name: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    load_character(character_name, state).await
}

/// Start the PoE OAuth PKCE flow — opens browser, waits for redirect, saves token.
#[tauri::command]
pub async fn start_oauth(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let token = oauth::start_oauth_flow()
        .await
        .map_err(|e| format!("OAuth failed: {e}"))?;

    state.db.save_oauth_token(&token)
        .map_err(|e| format!("Failed to save token: {e}"))?;

    Ok(format!("Connected! Scope: {}", token.scope))
}

/// Check whether the user has a valid OAuth token stored.
#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> bool {
    state.db.has_oauth_token()
}

/// List all saved builds (for history panel).
#[tauri::command]
pub fn list_builds(state: State<'_, AppState>) -> Result<Vec<BuildSummary>, String> {
    state.db.list_builds()
        .map_err(|e| format!("Failed to list builds: {e}"))
}

