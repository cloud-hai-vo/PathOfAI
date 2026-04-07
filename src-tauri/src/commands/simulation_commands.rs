use tauri::State;
use crate::AppState;
use crate::core::combat_sim::{
    simulate_map, PlayerState, DefenseSnapshot, OffenseSnapshot, Monster, FlaskState, SimResult,
};
use crate::core::build_comparator::{compare_builds, BuildSnapshot, BuildComparison};
use crate::core::stash::{tally_currency, StashItem, WealthSummary};
use crate::core::map_tracker::{accumulate_stats, MapRun, MapStats};
use crate::core::alert_manager::{check_alerts, deactivate_alert, PriceAlert, AlertFired};
use crate::core::map_mod_analyzer::{score_map_mods, MapDangerResult};
use crate::core::mana_reservation::{calculate_reservation, ReservationSkill, PlayerReservationStats, ReservationResult};
use crate::core::share_codec::{encode_share_code, decode_share_code, SharePayload};
use crate::core::stat_checker::{check_requirements, StatCheckResult};
use crate::core::vendor_recipe::{detect_recipes, RecipeCandidate, RecipeAnalysis};
use crate::core::ailment_mechanics::{
    calc_ignite, calc_chill, calc_freeze, calc_shock, calc_poison, calc_bleed,
    IgniteResult, ChillResult, FreezeResult, ShockResult, PoisonResult, BleedResult,
};
use crate::models::analysis::AnalysisResult;
use crate::models::build::Item;
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

/// Score a list of map mod texts against the current build's analysis.
/// Returns per-mod danger ratings and an overall verdict.
#[tauri::command]
pub async fn analyze_map_mods(
    map_mods_json:  String,   // Vec<String>
    analysis_json:  String,   // AnalysisResult
    _state: State<'_, AppState>,
) -> Result<MapDangerResult, String> {
    let mods: Vec<String>    = serde_json::from_str(&map_mods_json).map_err(|e| e.to_string())?;
    let analysis: AnalysisResult = serde_json::from_str(&analysis_json).map_err(|e| e.to_string())?;
    let mod_refs: Vec<&str>  = mods.iter().map(|s| s.as_str()).collect();
    Ok(score_map_mods(&mod_refs, &analysis))
}

/// Persist a new price alert to the database.
#[tauri::command]
pub async fn set_price_alert(
    alert_json: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let alert: PriceAlert = serde_json::from_str(&alert_json).map_err(|e| e.to_string())?;
    state.db.save_alert(&alert).map_err(|e| e.to_string())
}

/// List all price alerts stored in the database.
#[tauri::command]
pub async fn list_price_alerts(
    state: State<'_, AppState>,
) -> Result<Vec<PriceAlert>, String> {
    state.db.list_alerts().map_err(|e| e.to_string())
}

/// Remove a price alert from the database by its id.
#[tauri::command]
pub async fn remove_price_alert(
    alert_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id: i64 = alert_id.parse().map_err(|_| format!("Invalid alert id: {alert_id}"))?;
    state.db.remove_alert(id).map_err(|e| e.to_string())
}

/// Calculate mana reservation for a list of auras/skills.
#[tauri::command]
pub async fn calculate_mana_reservation(
    skills_json: String,
    player_json: String,
    _state: State<'_, AppState>,
) -> Result<ReservationResult, String> {
    let skills: Vec<ReservationSkill>     = serde_json::from_str(&skills_json).map_err(|e| e.to_string())?;
    let player: PlayerReservationStats    = serde_json::from_str(&player_json).map_err(|e| e.to_string())?;
    Ok(calculate_reservation(&skills, &player))
}

/// Generate a compact share code from a build payload.
#[tauri::command]
pub async fn generate_share_code(
    payload_json: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    let payload: SharePayload = serde_json::from_str(&payload_json).map_err(|e| e.to_string())?;
    encode_share_code(&payload).map_err(|e| e.to_string())
}

/// Decode a share code back into a build payload.
#[tauri::command]
pub async fn import_share_code(
    code: String,
    _state: State<'_, AppState>,
) -> Result<SharePayload, String> {
    decode_share_code(&code).map_err(|e| e.to_string())
}

/// Check stat requirements for all equipped items, optionally simulating a candidate swap.
#[tauri::command]
pub async fn check_stat_requirements(
    build_json:     String,
    candidate_json: Option<String>,
    _state: State<'_, AppState>,
) -> Result<StatCheckResult, String> {
    let build = serde_json::from_str(&build_json).map_err(|e| e.to_string())?;
    let candidate: Option<Item> = match candidate_json {
        Some(ref json) => Some(serde_json::from_str(json).map_err(|e| e.to_string())?),
        None => None,
    };
    Ok(check_requirements(&build, candidate.as_ref()))
}

/// Detect vendor recipes (chaos, regal, quality) from a list of stash items.
#[tauri::command]
pub async fn detect_vendor_recipes(
    items_json: String,
    _state: State<'_, AppState>,
) -> Result<RecipeAnalysis, String> {
    let items: Vec<RecipeCandidate> = serde_json::from_str(&items_json).map_err(|e| e.to_string())?;
    Ok(detect_recipes(&items))
}

/// Calculate ignite DPS and duration for a given hit.
#[tauri::command]
pub async fn calc_ignite_cmd(
    fire_hit: f64, fire_dot_multi_pct: f64, increased_burning_pct: f64,
    increased_duration_pct: f64, _state: State<'_, AppState>,
) -> Result<IgniteResult, String> {
    Ok(calc_ignite(fire_hit, fire_dot_multi_pct, increased_burning_pct, increased_duration_pct))
}

/// Calculate chill effect and duration.
#[tauri::command]
pub async fn calc_chill_cmd(
    cold_hit: f64, target_max_life: f64, increased_effect_pct: f64,
    increased_duration_pct: f64, _state: State<'_, AppState>,
) -> Result<ChillResult, String> {
    Ok(calc_chill(cold_hit, target_max_life, increased_effect_pct, increased_duration_pct))
}

/// Calculate freeze ability and duration.
#[tauri::command]
pub async fn calc_freeze_cmd(
    cold_hit: f64, target_max_life: f64, _state: State<'_, AppState>,
) -> Result<FreezeResult, String> {
    Ok(calc_freeze(cold_hit, target_max_life))
}

/// Calculate shock effect and duration.
#[tauri::command]
pub async fn calc_shock_cmd(
    lightning_hit: f64, target_max_life: f64, increased_effect_pct: f64,
    increased_duration_pct: f64, has_always_shocks: bool, _state: State<'_, AppState>,
) -> Result<ShockResult, String> {
    Ok(calc_shock(lightning_hit, target_max_life, increased_effect_pct, increased_duration_pct, has_always_shocks))
}

/// Calculate poison DPS and stack count.
#[tauri::command]
pub async fn calc_poison_cmd(
    phys_chaos_hit: f64, hit_rate: f64, poison_chance_pct: f64,
    chaos_dot_multi_pct: f64, increased_poison_pct: f64,
    increased_duration_pct: f64, _state: State<'_, AppState>,
) -> Result<PoisonResult, String> {
    Ok(calc_poison(phys_chaos_hit, hit_rate, poison_chance_pct, chaos_dot_multi_pct, increased_poison_pct, increased_duration_pct))
}

/// Calculate bleed DPS and stack count.
#[tauri::command]
pub async fn calc_bleed_cmd(
    phys_hit: f64, hit_rate: f64, bleed_chance_pct: f64, phys_dot_multi_pct: f64,
    increased_bleed_pct: f64, increased_duration_pct: f64,
    has_crimson_dance: bool, target_is_moving: bool, _state: State<'_, AppState>,
) -> Result<BleedResult, String> {
    Ok(calc_bleed(phys_hit, hit_rate, bleed_chance_pct, phys_dot_multi_pct,
        increased_bleed_pct, increased_duration_pct, has_crimson_dance, target_is_moving))
}
