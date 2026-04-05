use tauri::State;
use crate::AppState;
use crate::models::{Item, AnalysisResult};
use crate::core::{item_parser, item_scorer, build_detector, image_resolver::{ImageResolver, ImageRequest}};
use crate::calculator::what_if::{ImpactTable, ItemStatDelta, StatType};

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

/// Fast item swap estimate using the pre-computed ImpactTable (Algorithm 25).
///
/// Returns estimated DPS and life change for replacing `current_item_json`
/// with `new_item_json` on the current build.
///
/// Both `*_item_json` are serialised `ItemStatDelta` objects:
/// `{ "mods": [ ["FlatLife", 80], ["FireDotMulti", 20] ] }`
#[tauri::command]
pub fn estimate_item_swap(
    build_id:         String,
    new_item_json:    String,
    current_item_json: String,
    state:            State<'_, AppState>,
) -> Result<EstimateResult, String> {
    let build = state.db.load_build(&build_id)
        .map_err(|e| format!("Build not found: {e}"))?;

    let analysis = state.db.load_analysis(&build_id)
        .map_err(|e| format!("Analysis not found: {e}"))?;

    let new_item: Vec<(String, f64)> = serde_json::from_str(&new_item_json)
        .map_err(|e| format!("Invalid new_item_json: {e}"))?;
    let cur_item: Vec<(String, f64)> = serde_json::from_str(&current_item_json)
        .map_err(|e| format!("Invalid current_item_json: {e}"))?;

    let new_delta  = parse_item_stat_delta(new_item)?;
    let cur_delta  = parse_item_stat_delta(cur_item)?;

    let table = ImpactTable::build(
        analysis.offense.total_dps,
        analysis.defenses.life as f64,
        &build,
    );
    let est = table.estimate_swap(&new_delta, &cur_delta);

    Ok(EstimateResult {
        dps_change:  est.dps_change,
        life_change: est.life_change,
        is_estimate: est.is_estimate,
    })
}

#[derive(serde::Serialize)]
pub struct EstimateResult {
    pub dps_change:  f64,
    pub life_change: f64,
    pub is_estimate: bool,
}

fn parse_item_stat_delta(pairs: Vec<(String, f64)>) -> Result<ItemStatDelta, String> {
    let mut mods = Vec::new();
    for (stat_name, value) in pairs {
        let stat = parse_stat_type(&stat_name)?;
        mods.push((stat, value));
    }
    Ok(ItemStatDelta { mods })
}

fn parse_stat_type(s: &str) -> Result<StatType, String> {
    match s {
        "FlatLife"        => Ok(StatType::FlatLife),
        "PercentLife"     => Ok(StatType::PercentLife),
        "FireDotMulti"    => Ok(StatType::FireDotMulti),
        "FlatPhysDamage"  => Ok(StatType::FlatPhysDamage),
        "AttackSpeed"     => Ok(StatType::AttackSpeed),
        "CritChance"      => Ok(StatType::CritChance),
        "CritMultiplier"  => Ok(StatType::CritMultiplier),
        "FireRes"         => Ok(StatType::FireRes),
        "ColdRes"         => Ok(StatType::ColdRes),
        "LightningRes"    => Ok(StatType::LightningRes),
        other             => Err(format!("Unknown stat type: {other}")),
    }
}

/// Resolve an item image URL — memory+disk cache, then CDN.
///
/// `item_type` is one of: "unique", "base", "gem", "currency"
/// `item_name` is the display name, e.g. "Kaom's Heart"
/// Returns a URL string suitable for `<img src="...">`.
#[tauri::command]
pub fn resolve_item_image(
    item_type: String,
    item_name: String,
    state:     State<'_, AppState>,
) -> String {
    let cache_dir = state.db.data_dir().join("cache").join("images");
    let mut resolver = ImageResolver::new(&cache_dir);

    let request = match item_type.as_str() {
        "unique"   => ImageRequest::UniqueItem(item_name),
        "base"     => ImageRequest::BaseType(item_name),
        "gem"      => ImageRequest::Gem(item_name),
        "currency" => ImageRequest::Currency(item_name),
        _          => ImageRequest::Placeholder,
    };

    resolver.resolve(&request).to_url()
}
