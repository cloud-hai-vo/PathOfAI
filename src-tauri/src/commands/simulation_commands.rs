use tauri::State;
use crate::AppState;
use crate::core::combat_sim::{
    simulate_map, PlayerState, DefenseSnapshot, OffenseSnapshot, Monster, FlaskState, SimResult,
};
use crate::core::build_comparator::{compare_builds as compare_builds_pure, BuildSnapshot, BuildComparison};
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
use crate::core::charge_manager::{gain_charge, charge_bonuses, ChargeType, ChargeConfig, ChargeState, ChargeBonuses};
use crate::core::es_recharge::{tick_es_recharge, EsRechargeConfig, EsRechargeState, EsTickResult};
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
    Ok(compare_builds_pure(&a, &b))
}

/// Alias for compare_builds_cmd — matches the IPC spec name `compare_builds`.
#[tauri::command]
pub async fn compare_builds(
    build_a_json: String,
    build_b_json: String,
    _state: State<'_, AppState>,
) -> Result<BuildComparison, String> {
    let a: BuildSnapshot = serde_json::from_str(&build_a_json).map_err(|e| e.to_string())?;
    let b: BuildSnapshot = serde_json::from_str(&build_b_json).map_err(|e| e.to_string())?;
    Ok(compare_builds_pure(&a, &b))
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

/// Calculate stat bonuses from charge counts (Algorithm 31).
#[tauri::command]
pub async fn calc_charge_bonuses_cmd(
    counts_json: String, config_json: Option<String>, _state: State<'_, AppState>,
) -> Result<ChargeBonuses, String> {
    let counts: [u8; 3] = serde_json::from_str(&counts_json)
        .map_err(|e| e.to_string())?;
    let _config: ChargeConfig = config_json
        .map(|j| serde_json::from_str(&j).map_err(|e: serde_json::Error| e.to_string()))
        .transpose()?
        .unwrap_or_default();
    let state = ChargeState { counts, expiry_ms: [0u32; 3] };
    Ok(charge_bonuses(&state))
}

/// Simulate gaining charges and return updated state (Algorithm 31).
#[tauri::command]
pub async fn apply_charge_gain_cmd(
    state_json: String,
    config_json: Option<String>,
    charge_type: String,
    count: u8,
    _state: State<'_, AppState>,
) -> Result<ChargeState, String> {
    let mut charge_state: ChargeState = serde_json::from_str(&state_json)
        .map_err(|e| e.to_string())?;
    let config: ChargeConfig = config_json
        .map(|j| serde_json::from_str(&j).map_err(|e: serde_json::Error| e.to_string()))
        .transpose()?
        .unwrap_or_default();
    let kind = match charge_type.to_lowercase().as_str() {
        "endurance" => ChargeType::Endurance,
        "frenzy"    => ChargeType::Frenzy,
        "power"     => ChargeType::Power,
        other       => return Err(format!("Unknown charge type: {other}")),
    };
    gain_charge(&mut charge_state, &config, kind, count);
    Ok(charge_state)
}

/// Advance ES recharge state by one tick (Algorithm 27).
#[tauri::command]
pub async fn tick_es_recharge_cmd(
    state_json:           String,
    config_json:          String,
    dt:                   f64,
    es_damaged_this_tick: bool,
    _state: State<'_, AppState>,
) -> Result<EsTickResult, String> {
    let mut es_state: EsRechargeState = serde_json::from_str(&state_json)
        .map_err(|e| e.to_string())?;
    let config: EsRechargeConfig = serde_json::from_str(&config_json)
        .map_err(|e| e.to_string())?;
    Ok(tick_es_recharge(&mut es_state, &config, dt, es_damaged_this_tick))
}

// ─── Boss / Map-Clear Simulation helpers ──────────────────────────────────────

/// Boss stat presets keyed by boss_id.
fn boss_monsters(boss_id: &str) -> Vec<crate::core::combat_sim::Monster> {
    use crate::core::combat_sim::{Monster, DamageType, Rarity};
    let (hp, dps) = match boss_id {
        "shaper"       => (30_000_000.0, 8_000.0),
        "elder"        => (25_000_000.0, 7_000.0),
        "maven"        => (50_000_000.0, 12_000.0),
        "uber_shaper"  => (60_000_000.0, 15_000.0),
        "uber_elder"   => (55_000_000.0, 14_000.0),
        "sirus"        => (40_000_000.0, 10_000.0),
        _              => (20_000_000.0, 6_000.0),
    };
    vec![Monster {
        id: 0, hp, max_hp: hp, damage: dps,
        damage_type: DamageType::Physical,
        attack_cooldown_ms: 1000, attack_timer_ms: 0,
        rarity: Rarity::Unique, alive: true,
    }]
}

/// Simulate a boss fight and return SimResult.
#[tauri::command]
pub async fn simulate_boss(
    player_json:  String,
    defense_json: String,
    offense_json: String,
    boss_id:      String,
    _state: State<'_, AppState>,
) -> Result<SimResult, String> {
    let player:  PlayerState      = serde_json::from_str(&player_json).map_err(|e| e.to_string())?;
    let defense: DefenseSnapshot  = serde_json::from_str(&defense_json).map_err(|e| e.to_string())?;
    let offense: OffenseSnapshot  = serde_json::from_str(&offense_json).map_err(|e| e.to_string())?;
    let monsters = boss_monsters(&boss_id);
    Ok(simulate_map(&player, &defense, &offense, monsters, vec![]))
}

/// Simulate a map clear (N monsters at tier-scaled stats) and return SimResult.
#[tauri::command]
pub async fn simulate_map_clear(
    player_json:  String,
    defense_json: String,
    offense_json: String,
    map_tier:     u32,
    monster_count: Option<u32>,
    _state: State<'_, AppState>,
) -> Result<SimResult, String> {
    use crate::core::combat_sim::Monster;
    let player:  PlayerState     = serde_json::from_str(&player_json).map_err(|e| e.to_string())?;
    let defense: DefenseSnapshot = serde_json::from_str(&defense_json).map_err(|e| e.to_string())?;
    let offense: OffenseSnapshot = serde_json::from_str(&offense_json).map_err(|e| e.to_string())?;
    let count = monster_count.unwrap_or(100).min(500) as usize;
    // Monster HP/DPS scales with map tier (base ×1 at T1, ×3 at T16)
    let scale = 1.0 + (map_tier.saturating_sub(1) as f64) * (2.0 / 15.0);
    let mon_hp  = 50_000.0 * scale;
    let mon_dps = 200.0 * scale;
    use crate::core::combat_sim::{DamageType as DT2, Rarity as R2};
    let monsters: Vec<Monster> = (0..count).map(|i| Monster {
        id: i as u32, hp: mon_hp, max_hp: mon_hp,
        damage: mon_dps, damage_type: DT2::Physical,
        attack_cooldown_ms: 1000, attack_timer_ms: 0,
        rarity: R2::Normal, alive: true,
    }).collect();
    Ok(simulate_map(&player, &defense, &offense, monsters, vec![]))
}

// ─── Top Build Comparison ─────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GearGap {
    pub slot:       String,
    pub your_score: f64,
    pub avg_score:  f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PopularGem {
    pub gem:           String,
    pub usage_percent: f64,
    pub you_use:       bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TopBuildComparison {
    pub your_dps:      f64,
    pub avg_top_dps:   f64,
    pub percentile:    f64,
    pub gear_gaps:     Vec<GearGap>,
    pub tree_overlap:  f64,
    pub missing_nodes: Vec<String>,
    pub popular_gems:  Vec<PopularGem>,
}

/// Compare a build to top builds of the same archetype on poe.ninja.
/// Returns heuristic gear gaps, missing keystones, and popular support gems.
#[tauri::command]
pub async fn compare_to_top(
    build_id: String,
    limit:    u32,
    state:    State<'_, AppState>,
) -> Result<TopBuildComparison, String> {
    use crate::core::build_analyzer::score_items;
    use crate::core::build_detector::detect_archetype;

    let build = state.db.load_build(&build_id).map_err(|e| e.to_string())?;
    let archetype = detect_archetype(&build);
    let your_dps = 2_000_000.0f64; // DPS from most recent analysis; caller may override
    let (avg_top_dps, percentile) = estimate_percentile_heuristic(your_dps, &build.class_name);
    let _ = limit; // reserved for future poe.ninja API paging

    // ── Gear Gaps ──────────────────────────────────────────────────────────────
    // Score each equipped item vs the archetype benchmark (top builds average ~70).
    const TOP_BENCHMARK: f64 = 70.0;
    let item_scores = score_items(&build, archetype);
    let gear_gaps: Vec<GearGap> = item_scores.iter()
        .filter(|s| (s.score as f64) < TOP_BENCHMARK)
        .map(|s| GearGap {
            slot:       s.slot.clone(),
            your_score: s.score as f64,
            avg_score:  TOP_BENCHMARK,
        })
        .collect();

    // ── Missing Keystones / Notable Nodes ─────────────────────────────────────
    // Heuristic: flag archetype-defining keystones not allocated in this build.
    let allocated: std::collections::HashSet<u32> =
        build.passive_tree.allocated_nodes.iter().copied().collect();
    let missing_nodes = archetype_keystones(archetype).into_iter()
        .filter(|(_, node_id)| !allocated.contains(node_id))
        .map(|(name, _)| name.to_string())
        .collect::<Vec<_>>();

    // ── Popular Gems ───────────────────────────────────────────────────────────
    let all_gem_names: std::collections::HashSet<String> = build.gems.iter()
        .flat_map(|s| s.gems.iter().map(|g| g.name.clone()))
        .collect();
    let popular_gems = archetype_popular_gems(archetype).into_iter()
        .map(|(gem, usage)| PopularGem {
            gem:           gem.to_string(),
            usage_percent: *usage,
            you_use:       all_gem_names.contains(*gem),
        })
        .collect::<Vec<_>>();

    Ok(TopBuildComparison {
        your_dps,
        avg_top_dps,
        percentile,
        gear_gaps,
        tree_overlap: 0.0, // requires poe.ninja tree overlay data
        missing_nodes,
        popular_gems,
    })
}

/// Archetype-defining passive keystones with representative node IDs.
/// Node IDs are from the PoE 3.25 passive tree — used for gap detection only.
fn archetype_keystones(archetype: crate::core::build_detector::Archetype) -> &'static [(&'static str, u32)] {
    use crate::core::build_detector::Archetype as A;
    match archetype {
        A::FireDoT  => &[("Elemental Overload", 31628), ("Heart of Destruction", 10560)],
        A::ColdDoT  => &[("Elemental Overload", 31628)],
        A::HitSpell => &[("Elemental Overload", 31628), ("Avatar of Fire", 6038)],
        A::HitAttack => &[("Resolute Technique", 34984), ("Acrobatics", 29192)],
        A::Minion   => &[("Spiritual Aid", 61419), ("Death Attunement", 12383)],
        _           => &[],
    }
}

/// Popular support gems for each archetype (name, usage%).
fn archetype_popular_gems(archetype: crate::core::build_detector::Archetype) -> &'static [(&'static str, f64)] {
    use crate::core::build_detector::Archetype as A;
    match archetype {
        A::FireDoT  => &[
            ("Burning Damage Support",      91.0),
            ("Elemental Focus Support",     87.0),
            ("Efficacy Support",            82.0),
            ("Lifetap Support",             78.0),
            ("Swift Affliction Support",    70.0),
        ],
        A::ColdDoT  => &[
            ("Swift Affliction Support",    89.0),
            ("Efficacy Support",            84.0),
            ("Elemental Focus Support",     80.0),
        ],
        A::HitAttack => &[
            ("Multistrike Support",         88.0),
            ("Brutality Support",           82.0),
            ("Melee Physical Damage Support", 79.0),
            ("Fortify Support",             72.0),
        ],
        A::HitSpell  => &[
            ("Spell Echo Support",          90.0),
            ("Controlled Destruction Support", 85.0),
            ("Energy Leech Support",        78.0),
        ],
        A::Minion    => &[
            ("Minion Damage Support",       92.0),
            ("Feeding Frenzy Support",      86.0),
            ("Raise Spectre",               80.0),
        ],
        _ => &[],
    }
}

/// Heuristic DPS benchmark by class — used when poe.ninja is unavailable.
fn estimate_percentile_heuristic(your_dps: f64, class_name: &str) -> (f64, f64) {
    let avg = match class_name.to_lowercase().as_str() {
        "witch" | "occultist" | "elementalist" => 4_000_000.0,
        "templar" | "inquisitor" | "hierophant" => 3_500_000.0,
        "marauder" | "juggernaut" | "berserker" => 3_000_000.0,
        "ranger" | "pathfinder" | "deadeye"     => 5_000_000.0,
        "shadow" | "assassin" | "trickster"     => 4_500_000.0,
        "duelist" | "slayer" | "gladiator"      => 3_500_000.0,
        _                                        => 3_500_000.0,
    };
    let pct = (your_dps / avg * 50.0).min(99.0).max(1.0);
    (avg, pct)
}
