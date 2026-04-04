use tauri::State;
use crate::AppState;
use crate::models::market::{PriceResult, TradeResult};

/// Get current poe.ninja prices for a list of item names.
#[tauri::command]
pub async fn get_prices(
    item_names: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<PriceResult>, String> {
    crate::market::price_cache::get_prices(&item_names)
        .await
        .map_err(|e| format!("Price fetch failed: {e}"))
}

/// Search trade for upgrade candidates in a specific slot.
#[tauri::command]
pub async fn search_upgrades(
    slot: String,
    budget_div: f64,
    build_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TradeResult>, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    let analysis = state.db.load_analysis(&build_id)
        .map_err(|e| format!("Analysis not found: {e}"))?;

    crate::market::upgrade_finder::find_upgrades(&slot, &build, &analysis, budget_div)
        .await
        .map_err(|e| format!("Trade search failed: {e}"))
}
