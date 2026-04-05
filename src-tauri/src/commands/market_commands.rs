use tauri::State;
use crate::AppState;
use crate::models::market::{PriceResult, TradeResult};
use crate::core::buy_timing::{BuyRecommendation, PricePoint, LeaguePhase};

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

/// Generate a buy timing recommendation for an item.
///
/// `history_json` is a JSON array of `{ "price_divine": f64 }` objects,
/// oldest-first. The frontend supplies this from its own price history store
/// or from poe.ninja sparkline data.
///
/// `league_phase` is one of: "LaunchFrenzy", "CrashPeriod", "Stabilization",
/// "PeakEconomy", "LateLeague".
#[tauri::command]
pub fn get_buy_recommendation(
    item_key:     String,
    history_json: String,
    league_phase: String,
) -> Result<BuyRecommendation, String> {
    let history: Vec<PricePoint> = serde_json::from_str(&history_json)
        .map_err(|e| format!("Invalid history JSON: {e}"))?;

    let phase = parse_league_phase(&league_phase)?;
    Ok(crate::core::buy_timing::generate_buy_recommendation(&item_key, &history, phase))
}

fn parse_league_phase(s: &str) -> Result<LeaguePhase, String> {
    match s {
        "LaunchFrenzy"  => Ok(LeaguePhase::LaunchFrenzy),
        "CrashPeriod"   => Ok(LeaguePhase::CrashPeriod),
        "Stabilization" => Ok(LeaguePhase::Stabilization),
        "PeakEconomy"   => Ok(LeaguePhase::PeakEconomy),
        "LateLeague"    => Ok(LeaguePhase::LateLeague),
        other           => Err(format!("Unknown league phase: {other}")),
    }
}
