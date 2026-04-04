use tauri::State;
use crate::AppState;
use crate::models::seer::{SeerResponse, TreeAnalysis};
use crate::models::market::CraftSuggestion;
use crate::seer::router;
use crate::core::build_analyzer;

/// Ask The Seer a free-form question about the current build.
#[tauri::command]
pub async fn ask_seer(
    question: String,
    build_id: String,
    state: State<'_, AppState>,
) -> Result<SeerResponse, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    // Analysis is cached in DB — reload or re-run
    let analysis = state.db.load_analysis(&build_id)
        .map_err(|e| format!("Analysis not found: {e}"))?;

    router::route(&question, &build, &analysis)
        .await
        .map_err(|e| format!("Seer error: {e}"))
}

/// Get passive tree analysis with node recommendations.
#[tauri::command]
pub async fn get_tree_analysis(
    build_id: String,
    state: State<'_, AppState>,
) -> Result<TreeAnalysis, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    build_analyzer::analyze_tree(&build)
        .map_err(|e| format!("Tree analysis failed: {e}"))
}

/// Get crafting suggestions based on current build and available currency.
#[tauri::command]
pub async fn get_craft_suggestions(
    build_id: String,
    currency_json: String,           // JSON-encoded CurrencyInventory
    state: State<'_, AppState>,
) -> Result<Vec<CraftSuggestion>, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    let currency = serde_json::from_str(&currency_json)
        .map_err(|e| format!("Invalid currency data: {e}"))?;

    crate::core::craft_advisor::get_suggestions(&build, &currency)
        .map_err(|e| format!("Craft suggestion failed: {e}"))
}
