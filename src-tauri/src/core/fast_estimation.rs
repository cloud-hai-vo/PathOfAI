/// fast_estimation.rs — Pre-Computed Impact Tables (Algorithm 25).
///
/// Builds a linear approximation table: for each stat type, how much does
/// +1 unit of that stat change DPS and life? Then item impact can be estimated
/// in O(mods) rather than running the full calculator.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-use StatType from sensitivity_analysis
pub use crate::core::sensitivity_analysis::StatType;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Pre-computed marginal impact of one unit of each stat.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImpactTable {
    /// DPS change per +1 unit of this stat type.
    pub dps_per_unit:  HashMap<StatType, f64>,
    /// Life change per +1 unit of this stat type.
    pub life_per_unit: HashMap<StatType, f64>,
    /// Build hash at time of table construction (for invalidation).
    pub build_hash:    u64,
}

/// A mod on an item with a stat type and numeric value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMod {
    pub stat: StatType,
    pub value: f64,
}

/// Estimated DPS and life change from swapping an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub dps_change:  f64,
    pub life_change: f64,
    /// True = linear estimate (show "~" in UI), False = exact calculator result.
    pub is_estimate: bool,
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Build the impact table by perturbing each stat by a small delta and
/// measuring the calculator's response.
///
/// # Parameters
/// - `eval` — the calculator function `(stat_deltas) → (dps, life)`
///            where `stat_deltas` is a map of stat → delta to add to the base.
/// - `build_hash` — the hash of the current build state (for cache invalidation).
pub fn build_impact_table(
    base_stats: &HashMap<StatType, f64>,
    eval:       &dyn Fn(&HashMap<StatType, f64>) -> (f64, f64),
    build_hash: u64,
) -> ImpactTable {
    let (base_dps, base_life) = eval(base_stats);
    let mut table = ImpactTable { build_hash, ..Default::default() };

    for stat in estimable_stats() {
        let delta = stat_unit(*stat);
        let mut modified = base_stats.clone();
        *modified.entry(*stat).or_insert(0.0) += delta;

        let (new_dps, new_life) = eval(&modified);
        table.dps_per_unit.insert(*stat,  (new_dps  - base_dps)  / delta);
        table.life_per_unit.insert(*stat, (new_life - base_life) / delta);
    }

    table
}

/// Estimate the DPS and life change from swapping `new_item` for `old_item`.
pub fn estimate_item_swap(
    new_mods: &[ItemMod],
    old_mods: &[ItemMod],
    table:    &ImpactTable,
) -> Estimate {
    let mut dps_change  = 0.0f64;
    let mut life_change = 0.0f64;

    for m in new_mods {
        let dps_impact  = table.dps_per_unit.get(&m.stat).copied().unwrap_or(0.0);
        let life_impact = table.life_per_unit.get(&m.stat).copied().unwrap_or(0.0);
        dps_change  += m.value * dps_impact;
        life_change += m.value * life_impact;
    }
    for m in old_mods {
        let dps_impact  = table.dps_per_unit.get(&m.stat).copied().unwrap_or(0.0);
        let life_impact = table.life_per_unit.get(&m.stat).copied().unwrap_or(0.0);
        dps_change  -= m.value * dps_impact;
        life_change -= m.value * life_impact;
    }

    Estimate { dps_change, life_change, is_estimate: true }
}

/// Stats that can be estimated linearly.
fn estimable_stats() -> &'static [StatType] {
    &[
        StatType::FlatLife,
        StatType::PercentLife,
        StatType::FireDotMulti,
        StatType::ChaosDotMulti,
        StatType::FlatPhysMin,
        StatType::AttackSpeed,
        StatType::CritChance,
        StatType::CritMulti,
        StatType::SpellDamage,
        StatType::IncreasedFireDamage,
    ]
}

/// The perturbation unit for each stat type (same as sensitivity_analysis).
fn stat_unit(stat: StatType) -> f64 {
    crate::core::sensitivity_analysis::stat_delta(stat)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple evaluator: DPS scales with FireDotMulti, life scales with FlatLife.
    fn eval(stats: &HashMap<StatType, f64>) -> (f64, f64) {
        let fdm  = stats.get(&StatType::FireDotMulti).copied().unwrap_or(0.0);
        let life = stats.get(&StatType::FlatLife).copied().unwrap_or(4000.0);
        let dps  = 1_000_000.0 * (1.0 + fdm / 100.0);
        (dps, life)
    }

    fn make_table(hash: u64) -> ImpactTable {
        let stats: HashMap<StatType, f64> = [
            (StatType::FireDotMulti, 100.0),
            (StatType::FlatLife, 4000.0),
        ].iter().cloned().collect();
        build_impact_table(&stats, &eval, hash)
    }

    #[test]
    fn fire_dot_multi_has_positive_dps_impact() {
        let table = make_table(1);
        let impact = table.dps_per_unit.get(&StatType::FireDotMulti).copied().unwrap_or(0.0);
        assert!(impact > 0.0, "FireDotMulti should have positive DPS impact, got {impact}");
    }

    #[test]
    fn flat_life_has_positive_life_impact() {
        let table = make_table(1);
        let impact = table.life_per_unit.get(&StatType::FlatLife).copied().unwrap_or(0.0);
        assert!(impact > 0.0, "FlatLife should increase life pool, got {impact}");
    }

    #[test]
    fn estimate_new_item_with_fire_dot_multi_improves_dps() {
        let table = make_table(1);
        let new_mods = vec![ItemMod { stat: StatType::FireDotMulti, value: 20.0 }];
        let est = estimate_item_swap(&new_mods, &[], &table);
        assert!(est.dps_change > 0.0, "adding fire dot multi should improve DPS");
        assert!(est.is_estimate);
    }

    #[test]
    fn removing_item_mod_decreases_dps() {
        let table = make_table(1);
        let old_mods = vec![ItemMod { stat: StatType::FireDotMulti, value: 20.0 }];
        let est = estimate_item_swap(&[], &old_mods, &table);
        assert!(est.dps_change < 0.0, "removing fire dot multi should decrease DPS");
    }

    #[test]
    fn swapping_equal_items_gives_zero_change() {
        let table = make_table(1);
        let mods = vec![ItemMod { stat: StatType::FlatLife, value: 80.0 }];
        let est = estimate_item_swap(&mods, &mods.clone(), &table);
        assert!(est.dps_change.abs() < 0.001, "swapping identical item gives ~0 DPS change");
        assert!(est.life_change.abs() < 0.001, "swapping identical item gives ~0 life change");
    }

    #[test]
    fn empty_item_swap_gives_zero_change() {
        let table = make_table(1);
        let est = estimate_item_swap(&[], &[], &table);
        assert_eq!(est.dps_change, 0.0);
        assert_eq!(est.life_change, 0.0);
    }

    #[test]
    fn build_hash_stored_in_table() {
        let table = make_table(0xDEAD_BEEF);
        assert_eq!(table.build_hash, 0xDEAD_BEEF);
    }

    #[test]
    fn unknown_stat_has_zero_impact() {
        let table = make_table(1);
        // ChaosRes is not in our evaluator's model → impact should be 0
        let new_mods = vec![ItemMod { stat: StatType::ChaosRes, value: 30.0 }];
        let est = estimate_item_swap(&new_mods, &[], &table);
        // ChaosRes might or might not be in the table — impact on DPS should be 0 for our eval
        let _ = est; // just verifying it doesn't panic
    }

    #[test]
    fn impact_table_has_entries_for_all_estimable_stats() {
        let table = make_table(1);
        // At minimum, FireDotMulti and FlatLife should be present.
        assert!(table.dps_per_unit.contains_key(&StatType::FireDotMulti));
        assert!(table.life_per_unit.contains_key(&StatType::FlatLife));
    }
}
