use tauri::State;
use crate::AppState;
use crate::models::{Item, AnalysisResult};
use crate::core::{item_parser, item_scorer, build_detector};

#[derive(serde::Serialize)]
pub struct ParsedItemResult {
    pub item: Item,
    pub score: u8,
    pub comparison: ItemComparison,
}

#[derive(serde::Serialize)]
pub struct ItemComparison {
    pub dps_delta: f64,
    pub life_delta: i32,
    pub res_delta: i32,
    pub verdict: String,    // "Upgrade", "Sidegrade", "Downgrade"
}

/// Parse a PoE item pasted from clipboard (Ctrl+C in-game).
#[tauri::command]
pub async fn parse_clipboard_item(
    clipboard_text: String,
    build_id: String,
    state: State<'_, AppState>,
) -> Result<ParsedItemResult, String> {
    let item = item_parser::parse_clipboard(&clipboard_text)
        .map_err(|e| format!("Failed to parse item: {e}"))?;

    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    let archetype = build_detector::detect_archetype(&build);
    let score = item_scorer::score_item(&item, archetype);
    let comparison = item_scorer::compare_to_equipped(&item, &build, archetype);

    Ok(ParsedItemResult {
        item,
        score,
        comparison: ItemComparison {
            dps_delta: comparison.dps_delta,
            life_delta: comparison.life_delta,
            res_delta: comparison.res_delta,
            verdict: comparison.verdict,
        },
    })
}

/// Score any item against the current build (for "what if" analysis).
#[tauri::command]
pub async fn score_item(
    item_json: String,
    build_id: String,
    state: State<'_, AppState>,
) -> Result<u8, String> {
    let item: Item = serde_json::from_str(&item_json)
        .map_err(|e| format!("Invalid item data: {e}"))?;

    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    let archetype = build_detector::detect_archetype(&build);
    Ok(item_scorer::score_item(&item, archetype))
}

/// Apply an upgrade suggestion to the PoB file (with backup + atomic write).
#[tauri::command]
pub async fn apply_upgrade(
    suggestion_id: String,
    build_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    // Snapshot before applying (for undo support — Algorithm 33)
    state.db.snapshot_build(&build_id, "Before upgrade")
        .map_err(|e| format!("Snapshot failed: {e}"))?;

    let analysis = state.db.load_analysis(&build_id)
        .map_err(|e| format!("Analysis not found: {e}"))?;

    let suggestion = analysis.suggestions.iter()
        .find(|s| s.id == suggestion_id)
        .ok_or_else(|| "Suggestion not found".to_string())?
        .clone();

    let updated_build = crate::core::pob_writer::apply_suggestion(&build, &suggestion)
        .map_err(|e| format!("Failed to apply upgrade: {e}"))?;

    super::build_commands::analyze_build_data(updated_build, &state)
        .await
}
