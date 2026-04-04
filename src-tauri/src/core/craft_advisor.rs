/// Crafting advisor — see ALGORITHMS.md Algorithm 24 and Algorithm 47.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::models::build::BuildData;
use crate::models::market::{CraftSuggestion, CraftMethod, CraftVerdict};
use crate::calculator::formulas::geometric_99th_percentile;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CurrencyInventory {
    pub chaos: f64,
    pub divine: f64,
    pub exalted: f64,
    pub essence_count: u32,
    pub fossil_count: u32,
}

pub fn get_suggestions(
    build: &BuildData,
    currency: &CurrencyInventory,
) -> Result<Vec<CraftSuggestion>> {
    // TODO: populate from mod database + build analysis
    // Stub: return example suggestions
    Ok(vec![
        CraftSuggestion {
            method: CraftMethod::BenchCraft,
            target_mod: "Life".to_string(),
            probability: 1.0,
            attempts_99pct: 1,
            expected_cost_chaos: 2.0,
            dps_gain: 0.0,
            verdict: CraftVerdict::BestOption,
        },
    ])
}
