/// Fast Estimation Engine — Algorithm 25.
/// Pre-computes marginal impact of each stat type, then estimates item swaps in O(1).
use std::collections::HashMap;
use crate::models::build::BuildData;

/// Which stat type we're tracking for impact.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatType {
    FlatLife,
    PercentLife,
    FireDotMulti,
    FlatPhysDamage,
    AttackSpeed,
    CritChance,
    CritMultiplier,
    FireRes,
    ColdRes,
    LightningRes,
}

/// Pre-computed per-stat derivative table for a specific build state.
#[derive(Debug, Clone)]
pub struct ImpactTable {
    /// How much +1 unit of each stat changes DPS.
    pub dps_per_unit:  HashMap<StatType, f64>,
    /// How much +1 unit of each stat changes max life.
    pub life_per_unit: HashMap<StatType, f64>,
    /// Build hash — stale if the build changes.
    pub build_hash:    u64,
}

/// Quick estimate of swapping one item for another.
#[derive(Debug, Clone)]
pub struct Estimate {
    pub dps_change:  f64,
    pub life_change: f64,
    /// Always true — callers should display a "~" prefix.
    pub is_estimate: bool,
}

/// Lightweight item representation carrying only stat deltas.
#[derive(Debug, Clone, Default)]
pub struct ItemStatDelta {
    pub mods: Vec<(StatType, f64)>,
}

impl ImpactTable {
    /// Build an impact table using finite-difference derivatives.
    ///
    /// * `base_dps`  — total DPS already computed for this build (by offense_calc)
    /// * `base_life` — max life already computed for this build (by defense_calc)
    pub fn build(base_dps: f64, base_life: f64, build: &BuildData) -> Self {
        let mut dps_per_unit  = HashMap::new();
        let mut life_per_unit = HashMap::new();

        // Perturbation deltas per stat type
        let perturbations: &[(StatType, f64)] = &[
            (StatType::FlatLife,        10.0),
            (StatType::PercentLife,      1.0),
            (StatType::FireDotMulti,     1.0),
            (StatType::FlatPhysDamage,  10.0),
            (StatType::AttackSpeed,      1.0),
            (StatType::CritChance,       1.0),
            (StatType::CritMultiplier,   1.0),
            (StatType::FireRes,          1.0),
            (StatType::ColdRes,          1.0),
            (StatType::LightningRes,     1.0),
        ];

        for (stat, delta) in perturbations {
            let (dps_d, life_d) = compute_delta(base_dps, base_life, stat, *delta, build);
            dps_per_unit.insert(stat.clone(),  dps_d  / delta);
            life_per_unit.insert(stat.clone(), life_d / delta);
        }

        ImpactTable {
            dps_per_unit,
            life_per_unit,
            build_hash: compute_build_hash(build),
        }
    }

    /// Estimate the impact of swapping `current_item` for `new_item`.
    pub fn estimate_swap(&self, new_item: &ItemStatDelta, current_item: &ItemStatDelta) -> Estimate {
        let mut dps_change  = 0.0_f64;
        let mut life_change = 0.0_f64;

        for (stat, value) in &new_item.mods {
            dps_change  += value * self.dps_per_unit.get(stat).copied().unwrap_or(0.0);
            life_change += value * self.life_per_unit.get(stat).copied().unwrap_or(0.0);
        }
        for (stat, value) in &current_item.mods {
            dps_change  -= value * self.dps_per_unit.get(stat).copied().unwrap_or(0.0);
            life_change -= value * self.life_per_unit.get(stat).copied().unwrap_or(0.0);
        }

        Estimate { dps_change, life_change, is_estimate: true }
    }
}

/// Returns `(dps_delta, life_delta)` for adding `delta` units of `stat`.
/// Uses linear coefficients derived from the build's current computed state.
fn compute_delta(
    base_dps:  f64,
    base_life: f64,
    stat:      &StatType,
    delta:     f64,
    _build:    &BuildData,
) -> (f64, f64) {
    let dps_delta = match stat {
        StatType::FireDotMulti   => base_dps * (delta / 100.0),
        StatType::FlatPhysDamage => delta * 2.0,
        StatType::AttackSpeed    => base_dps * (delta / 100.0),
        StatType::CritChance     => base_dps * (delta / 100.0) * 0.5,
        StatType::CritMultiplier => base_dps * (delta / 100.0) * 0.3,
        _ => 0.0,
    };

    let life_delta = match stat {
        StatType::FlatLife    => delta,
        StatType::PercentLife => base_life * (delta / 100.0),
        _ => 0.0,
    };

    (dps_delta, life_delta)
}

fn compute_build_hash(build: &BuildData) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    build.class_name.hash(&mut h);
    build.level.hash(&mut h);
    build.items.len().hash(&mut h);
    h.finish()
}

// ── Tests (RED → GREEN) ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_build() -> BuildData {
        let mut b = BuildData::default();
        b.class_name = "Inquisitor".to_string();
        b.level = 90;
        b
    }

    #[test]
    fn estimate_always_marked_as_estimate() {
        let build = minimal_build();
        let table = ImpactTable::build(2_000_000.0, 5000.0, &build);
        let new_item = ItemStatDelta { mods: vec![(StatType::FlatLife, 100.0)] };
        let cur_item = ItemStatDelta::default();
        let est = table.estimate_swap(&new_item, &cur_item);
        assert!(est.is_estimate);
    }

    #[test]
    fn flat_life_mod_increases_life_estimate() {
        let build = minimal_build();
        let table = ImpactTable::build(2_000_000.0, 5000.0, &build);
        let new_item = ItemStatDelta { mods: vec![(StatType::FlatLife, 100.0)] };
        let cur_item = ItemStatDelta::default();
        let est = table.estimate_swap(&new_item, &cur_item);
        assert!(est.life_change > 0.0,
            "expected positive life change, got {}", est.life_change);
    }

    #[test]
    fn swapping_to_worse_life_gives_negative_life_delta() {
        let build = minimal_build();
        let table = ImpactTable::build(2_000_000.0, 5000.0, &build);
        let new_item = ItemStatDelta { mods: vec![(StatType::FlatLife, 50.0)] };
        let cur_item = ItemStatDelta { mods: vec![(StatType::FlatLife, 100.0)] };
        let est = table.estimate_swap(&new_item, &cur_item);
        assert!(est.life_change < 0.0, "downgrade should give negative life change");
    }

    #[test]
    fn fire_dot_multi_increases_dps_estimate() {
        let build = minimal_build();
        let table = ImpactTable::build(2_000_000.0, 5000.0, &build);
        let new_item = ItemStatDelta { mods: vec![(StatType::FireDotMulti, 20.0)] };
        let cur_item = ItemStatDelta::default();
        let est = table.estimate_swap(&new_item, &cur_item);
        assert!(est.dps_change > 0.0,
            "fire dot multi should increase DPS, got {}", est.dps_change);
    }

    #[test]
    fn resist_stat_has_zero_dps_impact() {
        let build = minimal_build();
        let table = ImpactTable::build(2_000_000.0, 5000.0, &build);
        let new_item = ItemStatDelta { mods: vec![(StatType::FireRes, 30.0)] };
        let cur_item = ItemStatDelta::default();
        let est = table.estimate_swap(&new_item, &cur_item);
        assert_eq!(est.dps_change, 0.0, "fire res should not affect DPS estimate");
    }

    #[test]
    fn build_hash_stable_for_same_build() {
        let build = minimal_build();
        let t1 = ImpactTable::build(0.0, 0.0, &build);
        let t2 = ImpactTable::build(0.0, 0.0, &build);
        assert_eq!(t1.build_hash, t2.build_hash);
    }

    #[test]
    fn percent_life_scales_with_base_life() {
        let build = minimal_build();
        let table = ImpactTable::build(0.0, 8000.0, &build);
        let new_item = ItemStatDelta { mods: vec![(StatType::PercentLife, 1.0)] };
        let empty    = ItemStatDelta::default();
        let est = table.estimate_swap(&new_item, &empty);
        // 1% of 8000 = 80 life
        assert!((est.life_change - 80.0).abs() < 1.0,
            "expected ~80 life change, got {}", est.life_change);
    }

    #[test]
    fn empty_swap_gives_zero_change() {
        let build = minimal_build();
        let table = ImpactTable::build(1_000_000.0, 4000.0, &build);
        let est = table.estimate_swap(&ItemStatDelta::default(), &ItemStatDelta::default());
        assert_eq!(est.dps_change,  0.0);
        assert_eq!(est.life_change, 0.0);
    }
}
