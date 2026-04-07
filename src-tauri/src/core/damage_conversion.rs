/// damage_conversion.rs — Damage Conversion Graph Resolver (Algorithm 2).
///
/// Resolves PoE's DAG-based damage conversion with modifier inheritance.
/// Conversion order: physical → lightning → cold → fire → chaos (one-way only).
/// Converted damage inherits %increased and more multipliers from BOTH source and
/// destination types.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Physical,
    Lightning,
    Cold,
    Fire,
    Chaos,
}

/// Topological conversion order — physical first, chaos last.
pub const TYPE_ORDER: [DamageType; 5] = [
    DamageType::Physical,
    DamageType::Lightning,
    DamageType::Cold,
    DamageType::Fire,
    DamageType::Chaos,
];

/// A single conversion edge: from_type → to_type at `pct` (0.0–1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionMod {
    pub from: DamageType,
    pub to:   DamageType,
    pub pct:  f64,  // 0.0–1.0
}

/// Gained-as-extra: adds damage without reducing the source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GainedAsExtra {
    pub from: DamageType,
    pub to:   DamageType,
    pub pct:  f64,  // 0.0–1.0
}

/// All inputs for one resolution pass.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversionInput {
    /// Base damage per type (flat added, before conversion).
    pub base:         HashMap<DamageType, f64>,
    /// Conversion mods — percentage expressed as 0.0–1.0.
    pub conversions:  Vec<ConversionMod>,
    /// Gained-as-extra mods.
    pub gained_extra: Vec<GainedAsExtra>,
    /// % increased per type (e.g., 100.0 = "+100% increased fire damage").
    pub increased:    HashMap<DamageType, f64>,
    /// More multipliers per type — each entry is a separate multiplicative layer.
    pub more:         HashMap<DamageType, Vec<f64>>,
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Resolve all conversions and return final damage per type after scaling.
///
/// Implements Algorithm 2: topological-order resolution with modifier inheritance.
pub fn resolve_conversion(input: &ConversionInput) -> HashMap<DamageType, f64> {
    // Step 1 — Normalize: cap total conversions out of each source type at 100%.
    let mut convs = input.conversions.clone();
    for &src in &TYPE_ORDER {
        let total: f64 = convs.iter().filter(|c| c.from == src).map(|c| c.pct).sum();
        if total > 1.0 {
            let scale = 1.0 / total;
            for c in convs.iter_mut().filter(|c| c.from == src) {
                c.pct *= scale;
            }
        }
    }

    // Step 2 — Process in topological order.
    //   pool[type]     = damage amount of that type (before increased/more)
    //   inherited[type] = set of source types this damage was converted from
    let mut pool: HashMap<DamageType, f64> = HashMap::new();
    let mut inherited: HashMap<DamageType, HashSet<DamageType>> = HashMap::new();

    for &t in &TYPE_ORDER {
        pool.insert(t, input.base.get(&t).copied().unwrap_or(0.0));
        inherited.insert(t, HashSet::from([t]));
    }

    for &src in &TYPE_ORDER {
        let src_pool  = pool.get(&src).copied().unwrap_or(0.0);
        if src_pool == 0.0 { continue; }
        let src_tags  = inherited.get(&src).unwrap().clone();

        // Apply conversion edges FROM this source.
        let mut remaining = 1.0f64;
        for c in convs.iter().filter(|c| c.from == src) {
            let amount = src_pool * c.pct;
            *pool.entry(c.to).or_insert(0.0) += amount;
            let dest_tags = inherited.entry(c.to).or_default();
            for &tag in &src_tags { dest_tags.insert(tag); }
            remaining -= c.pct;
        }

        // Apply gained-as-extra FROM this source (non-destructive — does NOT reduce source).
        for g in input.gained_extra.iter().filter(|g| g.from == src) {
            let amount = src_pool * g.pct;
            *pool.entry(g.to).or_insert(0.0) += amount;
            let dest_tags = inherited.entry(g.to).or_default();
            for &tag in &src_tags { dest_tags.insert(tag); }
        }

        // Reduce source by converted fraction.
        *pool.entry(src).or_insert(0.0) = src_pool * remaining.max(0.0);
    }

    // Step 3 — Apply increased/more using inherited tags.
    let mut result: HashMap<DamageType, f64> = HashMap::new();
    for &t in &TYPE_ORDER {
        let mut val = pool.get(&t).copied().unwrap_or(0.0);
        if val == 0.0 {
            result.insert(t, 0.0);
            continue;
        }

        // Sum %increased from all inherited source types.
        let mut total_inc = 0.0f64;
        for &tag in inherited.get(&t).unwrap() {
            total_inc += input.increased.get(&tag).copied().unwrap_or(0.0);
        }
        val *= 1.0 + total_inc / 100.0;

        // Apply each more multiplier from inherited source types.
        for &tag in inherited.get(&t).unwrap() {
            if let Some(mores) = input.more.get(&tag) {
                for &m in mores {
                    val *= 1.0 + m / 100.0;
                }
            }
        }

        result.insert(t, val);
    }

    result
}

/// Convenience: sum all damage types in a result map.
pub fn total_damage(result: &HashMap<DamageType, f64>) -> f64 {
    result.values().sum()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn base(phys: f64) -> ConversionInput {
        let mut i = ConversionInput::default();
        i.base.insert(DamageType::Physical, phys);
        i
    }

    fn cv(from: DamageType, to: DamageType, pct: f64) -> ConversionMod {
        ConversionMod { from, to, pct }
    }
    fn ex(from: DamageType, to: DamageType, pct: f64) -> GainedAsExtra {
        GainedAsExtra { from, to, pct }
    }

    fn get(r: &HashMap<DamageType, f64>, t: DamageType) -> f64 {
        r.get(&t).copied().unwrap_or(0.0)
    }

    // Test 1: No conversion — base passes through unchanged.
    #[test]
    fn no_conversion_returns_base_unchanged() {
        let result = resolve_conversion(&base(100.0));
        assert!((get(&result, DamageType::Physical) - 100.0).abs() < 0.001);
        assert!(get(&result, DamageType::Fire) < 0.001);
    }

    // Test 2: Simple 50% phys → fire.
    #[test]
    fn simple_50pct_phys_to_fire() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical, DamageType::Fire, 0.5));
        let r = resolve_conversion(&i);
        assert!((get(&r, DamageType::Physical) - 50.0).abs() < 0.001, "50 phys remaining");
        assert!((get(&r, DamageType::Fire)     - 50.0).abs() < 0.001, "50 fire converted");
    }

    // Test 3: Over-cap (60% fire + 60% cold = 120% → scaled to 50/50).
    #[test]
    fn over_cap_conversion_scales_proportionally() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical, DamageType::Fire, 0.6));
        i.conversions.push(cv(DamageType::Physical, DamageType::Cold, 0.6));
        let r = resolve_conversion(&i);
        assert!(get(&r, DamageType::Physical) < 0.001, "all phys converted");
        assert!((get(&r, DamageType::Fire) - 50.0).abs() < 0.001, "50 fire");
        assert!((get(&r, DamageType::Cold) - 50.0).abs() < 0.001, "50 cold");
    }

    // Test 4: Full conversion phys → light → cold → fire.
    #[test]
    fn chain_conversion_100pct() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical,  DamageType::Lightning, 1.0));
        i.conversions.push(cv(DamageType::Lightning, DamageType::Cold,      1.0));
        i.conversions.push(cv(DamageType::Cold,      DamageType::Fire,      1.0));
        let r = resolve_conversion(&i);
        assert!(get(&r, DamageType::Physical)  < 0.001, "no phys");
        assert!(get(&r, DamageType::Lightning) < 0.001, "no lightning");
        assert!(get(&r, DamageType::Cold)      < 0.001, "no cold");
        assert!((get(&r, DamageType::Fire) - 100.0).abs() < 0.001, "100 fire");
    }

    // Test 5: Chain + phys modifier inherited all the way to fire.
    #[test]
    fn chain_conversion_inherits_phys_modifier() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical,  DamageType::Lightning, 1.0));
        i.conversions.push(cv(DamageType::Lightning, DamageType::Cold,      1.0));
        i.conversions.push(cv(DamageType::Cold,      DamageType::Fire,      1.0));
        i.increased.insert(DamageType::Physical, 100.0); // +100% inc phys
        let r = resolve_conversion(&i);
        // Fire inherits phys tag → 100 * (1 + 1.0) = 200
        assert!((get(&r, DamageType::Fire) - 200.0).abs() < 0.001,
            "fire inherits phys +100%, expected 200, got {}", get(&r, DamageType::Fire));
    }

    // Test 6: Gained-as-extra is non-destructive.
    #[test]
    fn gained_as_extra_does_not_reduce_source() {
        let mut i = base(100.0);
        i.gained_extra.push(ex(DamageType::Physical, DamageType::Fire, 0.20));
        let r = resolve_conversion(&i);
        assert!((get(&r, DamageType::Physical) - 100.0).abs() < 0.001, "phys unchanged");
        assert!((get(&r, DamageType::Fire)     - 20.0).abs()  < 0.001, "20 extra fire");
    }

    // Test 7: Mixed conversion + gained-as-extra.
    #[test]
    fn mixed_conversion_and_extra() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical, DamageType::Fire, 0.5));
        i.gained_extra.push(ex(DamageType::Physical, DamageType::Fire, 0.20));
        let r = resolve_conversion(&i);
        assert!((get(&r, DamageType::Physical) - 50.0).abs() < 0.001, "50 phys");
        // 50 converted + 20 extra = 70
        assert!((get(&r, DamageType::Fire) - 70.0).abs() < 0.001, "70 fire");
    }

    // Test 8: The Algorithm 2 worked example — modifier inheritance correctness.
    //   100 phys, 50% phys→fire, +100% inc phys, +100% inc fire
    //   Phys:  50 × (1 + 1.0) = 100
    //   Fire:  50 × (1 + 1.0 + 1.0) = 150   ← inherits phys + fire increased
    #[test]
    fn converted_fire_inherits_both_phys_and_fire_modifiers() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical, DamageType::Fire, 0.5));
        i.increased.insert(DamageType::Physical, 100.0);
        i.increased.insert(DamageType::Fire,     100.0);
        let r = resolve_conversion(&i);
        assert!((get(&r, DamageType::Physical) - 100.0).abs() < 0.001, "100 phys");
        assert!((get(&r, DamageType::Fire)     - 150.0).abs() < 0.001,
            "150 fire (inherits phys+fire), got {}", get(&r, DamageType::Fire));
    }

    // Test 9: More multipliers stack multiplicatively.
    #[test]
    fn more_multipliers_stack_multiplicatively() {
        let mut i = base(100.0);
        i.more.insert(DamageType::Physical, vec![50.0, 50.0]); // two 50% more
        let r = resolve_conversion(&i);
        // 100 * 1.5 * 1.5 = 225
        assert!((get(&r, DamageType::Physical) - 225.0).abs() < 0.001,
            "expected 225, got {}", get(&r, DamageType::Physical));
    }

    // Test 10: Zero base yields zero result.
    #[test]
    fn zero_base_returns_all_zeros() {
        let i = ConversionInput::default();
        let r = resolve_conversion(&i);
        assert!(total_damage(&r) < 0.001);
    }

    // Test 11: total_damage helper sums all types.
    #[test]
    fn total_damage_sums_all_types() {
        let mut i = ConversionInput::default();
        i.base.insert(DamageType::Physical, 60.0);
        i.base.insert(DamageType::Fire,     40.0);
        let r = resolve_conversion(&i);
        assert!((total_damage(&r) - 100.0).abs() < 0.001);
    }

    // Test 12: 100% phys → fire removes all phys.
    #[test]
    fn full_phys_conversion_leaves_no_phys() {
        let mut i = base(100.0);
        i.conversions.push(cv(DamageType::Physical, DamageType::Fire, 1.0));
        let r = resolve_conversion(&i);
        assert!(get(&r, DamageType::Physical) < 0.001);
        assert!((get(&r, DamageType::Fire) - 100.0).abs() < 0.001);
    }
}
