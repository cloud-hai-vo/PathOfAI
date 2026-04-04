use tauri::State;
use crate::AppState;
use crate::models::{AnalysisResult, BuildData};
use crate::core::{pob_parser, build_analyzer, build_detector};
use crate::calculator::{defense_calc, offense_calc};

/// Load and analyze a PoB XML file.
/// Primary import path when user doesn't use OAuth.
#[tauri::command]
pub async fn analyze_build(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let build = pob_parser::parse_file(&file_path)
        .map_err(|e| format!("Failed to parse PoB file: {e}"))?;

    analyze_build_data(build, &state).await
}

/// Re-analyze the currently loaded build (e.g. after external PoB edit).
#[tauri::command]
pub async fn refresh_analysis(
    build_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    analyze_build_data(build, &state).await
}

/// Undo the last applied upgrade (restore from snapshot).
#[tauri::command]
pub async fn undo_last_change(
    build_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let build = state.db.undo_snapshot(&build_id)
        .map_err(|e| format!("Undo failed: {e}"))?;

    analyze_build_data(build, &state).await
}

/// Redo a previously undone change.
#[tauri::command]
pub async fn redo_change(
    build_id: String,
    state: State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let build = state.db.redo_snapshot(&build_id)
        .map_err(|e| format!("Redo failed: {e}"))?;

    analyze_build_data(build, &state).await
}

// --- shared logic ---

pub async fn analyze_build_data(
    build: BuildData,
    state: &State<'_, AppState>,
) -> Result<AnalysisResult, String> {
    let archetype = build_detector::detect_archetype(&build);
    let defenses = defense_calc::calculate(&build);
    let offense = offense_calc::calculate(&build);
    let issues = build_analyzer::detect_issues(&build, &defenses, &offense);
    let item_scores = build_analyzer::score_items(&build, archetype);
    let suggestions = build_analyzer::generate_suggestions(&build, &issues, &item_scores);

    let overall_score = build_analyzer::overall_score(&defenses, &offense, &issues);

    let result = AnalysisResult {
        build_id: build.id.clone(),
        build_name: build.name.clone(),
        class_name: build.class_name.clone(),
        ascendancy: build.ascendancy.clone(),
        level: build.level,
        archetype: archetype.id().to_string(),
        archetype_label: archetype.label().to_string(),
        overall_score,
        defenses,
        offense,
        issues,
        suggestions,
        item_scores,
    };

    // Persist build + result to SQLite
    state.db.save_build(&build)
        .map_err(|e| format!("Failed to save build: {e}"))?;

    Ok(result)
}
