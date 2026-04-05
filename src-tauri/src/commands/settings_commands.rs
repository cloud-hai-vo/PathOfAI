/// Settings commands — configure AI providers, league, app preferences.
use tauri::State;
use crate::AppState;
use crate::core::cloud_ai::{CloudProvider, ConnectionTestResult};

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
