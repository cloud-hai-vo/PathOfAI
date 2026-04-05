use tauri::State;
use crate::AppState;
use crate::core::combat_sim::{
    simulate_map, PlayerState, DefenseSnapshot, OffenseSnapshot, Monster, FlaskState, SimResult,
};
use crate::core::build_comparator::{compare_builds, BuildSnapshot, BuildComparison};
use crate::core::stash::{tally_currency, StashItem, WealthSummary};
use crate::core::map_tracker::{accumulate_stats, MapRun, MapStats};
use crate::core::alert_manager::{check_alerts, deactivate_alert, PriceAlert, AlertFired};
use std::collections::HashMap;

/// Run combat simulation for a build.
#[tauri::command]
pub async fn run_simulation(
    player_json:   String,
    defense_json:  String,
    offense_json:  String,
    monsters_json: String,
    flasks_json:   String,
    _state: State<'_, AppState>,
) -> Result<SimResult, String> {
    let player:   PlayerState      = serde_json::from_str(&player_json).map_err(|e| e.to_string())?;
    let defense:  DefenseSnapshot  = serde_json::from_str(&defense_json).map_err(|e| e.to_string())?;
    let offense:  OffenseSnapshot  = serde_json::from_str(&offense_json).map_err(|e| e.to_string())?;
    let monsters: Vec<Monster>     = serde_json::from_str(&monsters_json).map_err(|e| e.to_string())?;
    let flasks:   Vec<FlaskState>  = serde_json::from_str(&flasks_json).map_err(|e| e.to_string())?;
    Ok(simulate_map(&player, &defense, &offense, monsters, flasks))
}

/// Compare two build snapshots.
#[tauri::command]
pub async fn compare_builds_cmd(
    build_a_json: String,
    build_b_json: String,
    _state: State<'_, AppState>,
) -> Result<BuildComparison, String> {
    let a: BuildSnapshot = serde_json::from_str(&build_a_json).map_err(|e| e.to_string())?;
    let b: BuildSnapshot = serde_json::from_str(&build_b_json).map_err(|e| e.to_string())?;
    Ok(compare_builds(&a, &b))
}

/// Tally stash tab wealth.
#[tauri::command]
pub async fn tally_stash_wealth(
    items_json:       String,
    divine_price_c:   f64,
    _state: State<'_, AppState>,
) -> Result<WealthSummary, String> {
    let items: Vec<StashItem> = serde_json::from_str(&items_json).map_err(|e| e.to_string())?;
    Ok(tally_currency(&items, divine_price_c))
}

/// Get map run statistics.
#[tauri::command]
pub async fn get_map_stats(
    runs_json: String,
    _state: State<'_, AppState>,
) -> Result<MapStats, String> {
    let runs: Vec<MapRun> = serde_json::from_str(&runs_json).map_err(|e| e.to_string())?;
    Ok(accumulate_stats(&runs))
}

/// Check all price alerts against current prices.
#[tauri::command]
pub async fn check_price_alerts(
    alerts_json: String,
    prices_json: String,
    _state: State<'_, AppState>,
) -> Result<Vec<AlertFired>, String> {
    let alerts: Vec<PriceAlert>          = serde_json::from_str(&alerts_json).map_err(|e| e.to_string())?;
    let prices: HashMap<String, f64>     = serde_json::from_str(&prices_json).map_err(|e| e.to_string())?;
    Ok(check_alerts(&alerts, &prices))
}

/// Deactivate a price alert by id.
#[tauri::command]
pub async fn deactivate_price_alert(
    alerts_json: String,
    alert_id:    String,
    _state: State<'_, AppState>,
) -> Result<Vec<PriceAlert>, String> {
    let mut alerts: Vec<PriceAlert> = serde_json::from_str(&alerts_json).map_err(|e| e.to_string())?;
    deactivate_alert(&mut alerts, &alert_id);
    Ok(alerts)
}
