/// Core PoE formulas — pure functions, no side effects.
/// All formula references: docs/ENGINE-DESIGN.md and docs/ALGORITHMS.md

/// Physical damage reduction from armour.
/// PoE formula: Reduction% = Armour / (Armour + 10 * Damage)
/// See Algorithm 14 (Effective HP Calculator)
pub fn armour_phys_reduction(armour: f64, hit_damage: f64) -> f64 {
    if armour <= 0.0 || hit_damage <= 0.0 {
        return 0.0;
    }
    let reduction = armour / (armour + 10.0 * hit_damage);
    reduction.min(0.9) // cap at 90%
}

/// Effective HP against a damage type considering reduction.
pub fn effective_hp(raw_hp: f64, reduction: f64) -> f64 {
    if reduction >= 1.0 { return f64::MAX; }
    raw_hp / (1.0 - reduction)
}

/// Evasion chance from evasion rating.
/// PoE formula: Chance = 1 - (Accuracy / (Accuracy + (Evasion/4)^0.8))
/// Simplified: if accuracy unknown, use 95% accuracy monster baseline
pub fn evasion_chance(evasion: f64, monster_accuracy: f64) -> f64 {
    if evasion <= 0.0 { return 0.0; }
    let chance = 1.0 - (monster_accuracy / (monster_accuracy + (evasion / 4.0_f64).powf(0.8)));
    chance.clamp(0.05, 0.95) // min 5% chance to be hit, max 95% evasion
}

/// Apply the PoE increased/more multiplier chain.
/// Base × (1 + sum_of_increased) × product_of_more_multipliers
pub fn apply_multiplier_chain(
    base: f64,
    increased_pct: f64,      // total % increased, e.g. 250.0 for +250%
    more_multipliers: &[f64], // each "more" as a factor, e.g. 1.44 for +44% more
) -> f64 {
    let after_increased = base * (1.0 + increased_pct / 100.0);
    more_multipliers.iter().fold(after_increased, |acc, &m| acc * m)
}

/// Resistance cap check — returns effective resistance (capped at max).
pub fn effective_resistance(raw: i32, max_res: i32) -> i32 {
    raw.min(max_res)
}

/// Damage taken after resistance.
pub fn damage_after_resistance(damage: f64, effective_res: i32) -> f64 {
    damage * (1.0 - effective_res as f64 / 100.0)
}

/// Geometric 99th percentile — attempts for 99% chance of success.
/// Formula: ceil(log(0.01) / log(1 - p))
/// See Algorithm 47 (Craft Suggestion Ranker)
pub fn geometric_99th_percentile(success_prob: f64) -> u32 {
    if success_prob <= 0.0 { return u32::MAX; }
    if success_prob >= 1.0 { return 1; }
    ((0.01_f64.ln()) / (1.0 - success_prob).ln()).ceil() as u32
}

/// Format DPS number for display (e.g. 2840000 → "2.84M")
pub fn format_dps(dps: f64) -> String {
    if dps >= 1_000_000.0 {
        format!("{:.2}M", dps / 1_000_000.0)
    } else if dps >= 1_000.0 {
        format!("{:.1}K", dps / 1_000.0)
    } else {
        format!("{:.0}", dps)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armour_reduction_standard_case() {
        // 10,000 armour vs 1,000 hit → 50% reduction
        let r = armour_phys_reduction(10_000.0, 1_000.0);
        assert!((r - 0.5).abs() < 0.001, "expected ~50%, got {r}");
    }

    #[test]
    fn armour_reduction_cap() {
        // Huge armour → capped at 90%
        let r = armour_phys_reduction(1_000_000.0, 1_000.0);
        assert_eq!(r, 0.9);
    }

    #[test]
    fn multiplier_chain_rf_example() {
        // Base: 1000, +234% increased, ×1.44 (Swift Affliction) × 1.19 (Elemental Focus)
        let dps = apply_multiplier_chain(1000.0, 234.0, &[1.44, 1.19]);
        // Expected: 1000 × 3.34 × 1.44 × 1.19 ≈ 5726
        assert!(dps > 5_000.0 && dps < 7_000.0, "expected ~5726, got {dps}");
    }

    #[test]
    fn geometric_99pct_deterministic() {
        // p=1.0 (benchcraft) → always 1 attempt
        assert_eq!(geometric_99th_percentile(1.0), 1);
    }

    #[test]
    fn geometric_99pct_coin_flip() {
        // p=0.5 → ceil(log(0.01)/log(0.5)) = ceil(6.64) = 7
        assert_eq!(geometric_99th_percentile(0.5), 7);
    }

    #[test]
    fn format_dps_millions() {
        assert_eq!(format_dps(2_840_000.0), "2.84M");
    }

    #[test]
    fn format_dps_thousands() {
        assert_eq!(format_dps(142_500.0), "142.5K");
    }
}
