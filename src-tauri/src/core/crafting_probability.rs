/// crafting_probability.rs — Crafting Probability Engine (Algorithm 15).
///
/// Computes the probability of hitting target mods via weighted sampling without
/// replacement from the mod pool, and estimates expected attempts and cost.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A mod in the crafting pool with its spawn weight and category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMod {
    pub id:        String,
    pub weight:    f64,
    pub is_prefix: bool,
}

/// A target mod requirement (must appear on the crafted item).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetMod {
    pub mod_id:    String,
    pub is_prefix: bool,
    /// Minimum tier requirement (1 = any tier, 2+ = T2 or better, etc.).
    pub min_tier:  u32,
    /// Fraction of total mod weight that meets the tier requirement (0.0–1.0).
    pub tier_weight_fraction: f64,
}

impl TargetMod {
    pub fn new(mod_id: &str, is_prefix: bool) -> Self {
        Self { mod_id: mod_id.into(), is_prefix, min_tier: 1, tier_weight_fraction: 1.0 }
    }
}

/// Input for a probability calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftProbInput {
    pub pool:    Vec<PoolMod>,
    pub targets: Vec<TargetMod>,
    /// Number of prefix/suffix slots (normally 3 each for rare items).
    pub num_prefix_slots: u32,
    pub num_suffix_slots: u32,
    /// Cost per crafting attempt in chaos orbs.
    pub cost_per_attempt: f64,
}

/// Output of the probability calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftProbResult {
    /// Per-attempt probability of getting ALL target mods.
    pub probability:       f64,
    /// Expected number of attempts to succeed.
    pub expected_attempts: u32,
    /// Expected cost in chaos orbs (expected_attempts × cost_per_attempt).
    pub expected_cost:     f64,
    /// 99th-percentile attempt count (geometric distribution).
    pub attempts_99pct:    u32,
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Calculate P(all target mods appear) for a single craft attempt.
///
/// Uses weighted sampling without replacement (hypergeometric approximation).
/// Targets are split into prefix and suffix groups; each group is sampled
/// independently from their respective pools.
pub fn calc_probability(input: &CraftProbInput) -> CraftProbResult {
    let prefix_pool: Vec<&PoolMod> = input.pool.iter().filter(|m| m.is_prefix).collect();
    let suffix_pool: Vec<&PoolMod> = input.pool.iter().filter(|m| !m.is_prefix).collect();

    let target_prefixes: Vec<&TargetMod> = input.targets.iter().filter(|t| t.is_prefix).collect();
    let target_suffixes: Vec<&TargetMod> = input.targets.iter().filter(|t| !t.is_prefix).collect();

    // P(all targets appear) = P(prefix targets) × P(suffix targets)
    let p_prefix = exact_probability(&target_prefixes, &prefix_pool, input.num_prefix_slots);
    let p_suffix = exact_probability(&target_suffixes, &suffix_pool, input.num_suffix_slots);

    // Apply tier weight fractions.
    let mut prob = p_prefix * p_suffix;
    for t in &input.targets {
        prob *= t.tier_weight_fraction;
    }

    // Clamp to valid probability range.
    prob = prob.clamp(0.0, 1.0);

    let expected_attempts = if prob > 0.0 { (1.0 / prob).ceil() as u32 } else { u32::MAX };
    let expected_cost     = expected_attempts as f64 * input.cost_per_attempt;
    let attempts_99pct    = geometric_99th_percentile(prob);

    CraftProbResult { probability: prob, expected_attempts, expected_cost, attempts_99pct }
}

/// P(all target mods appear in `num_slots` weighted rolls without replacement).
///
/// For each target mod, we compute P(it appears in at least one of the remaining
/// slots) assuming the previous targets have already consumed slots.
fn exact_probability(targets: &[&TargetMod], pool: &[&PoolMod], num_slots: u32) -> f64 {
    if targets.is_empty() { return 1.0; }
    if pool.is_empty()    { return 0.0; }

    let mut total_weight: f64 = pool.iter().map(|m| m.weight).sum();
    if total_weight <= 0.0 { return 0.0; }

    let mut prob = 1.0f64;
    let mut slots_remaining = num_slots;

    for target in targets {
        if slots_remaining == 0 { return 0.0; }

        // Find the target mod's weight in the pool.
        let target_weight = pool.iter()
            .find(|m| m.id == target.mod_id)
            .map(|m| m.weight)
            .unwrap_or(0.0);

        if target_weight <= 0.0 { return 0.0; }

        // P(this mod appears in at least one of slots_remaining rolls).
        // Approximation: P = 1 − ((W - w_target) / W)^slots_remaining
        let p_miss = ((total_weight - target_weight) / total_weight)
            .max(0.0)
            .powi(slots_remaining as i32);

        prob *= 1.0 - p_miss;

        // Consume: this mod takes one slot.
        total_weight   -= target_weight;
        slots_remaining = slots_remaining.saturating_sub(1);
    }

    prob
}

/// 99th-percentile attempt count for a geometric distribution with success probability p.
/// P(succeed within N attempts) = 1 − (1-p)^N ≥ 0.99
/// N = ceil(ln(0.01) / ln(1-p))
pub fn geometric_99th_percentile(p: f64) -> u32 {
    if p <= 0.0 { return u32::MAX; }
    if p >= 1.0 { return 1; }
    let n = (0.01f64.ln() / (1.0 - p).ln()).ceil() as u32;
    n.max(1)
}

/// Expected attempts = 1/p (geometric distribution mean).
pub fn geometric_mean(p: f64) -> u32 {
    if p <= 0.0 { return u32::MAX; }
    if p >= 1.0 { return 1; }
    (1.0 / p).ceil() as u32
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(mods: &[(&str, f64, bool)]) -> Vec<PoolMod> {
        mods.iter().map(|(id, w, pfx)| PoolMod { id: id.to_string(), weight: *w, is_prefix: *pfx }).collect()
    }

    fn input(pool: Vec<PoolMod>, targets: Vec<TargetMod>, pfx_slots: u32, sfx_slots: u32, cost: f64) -> CraftProbInput {
        CraftProbInput { pool, targets, num_prefix_slots: pfx_slots, num_suffix_slots: sfx_slots, cost_per_attempt: cost }
    }

    #[test]
    fn single_mod_in_pool_of_one_is_certainty() {
        // Only mod in the prefix pool, 1 prefix slot — must appear.
        let p = pool(&[("life", 100.0, true)]);
        let t = vec![TargetMod::new("life", true)];
        let result = calc_probability(&input(p, t, 1, 3, 1.0));
        assert!((result.probability - 1.0).abs() < 0.001, "p should be 1.0");
        assert_eq!(result.expected_attempts, 1);
    }

    #[test]
    fn target_missing_from_pool_gives_zero_probability() {
        let p = pool(&[("resist", 100.0, true)]);
        let t = vec![TargetMod::new("life", true)]; // "life" not in pool
        let result = calc_probability(&input(p, t, 3, 3, 1.0));
        assert_eq!(result.probability, 0.0);
    }

    #[test]
    fn two_equal_weight_mods_one_target_gives_roughly_half() {
        // 2 mods, equal weight, 1 slot → P(target appears) ≈ 0.5
        let p = pool(&[("life", 100.0, true), ("resist", 100.0, true)]);
        let t = vec![TargetMod::new("life", true)];
        let result = calc_probability(&input(p, t, 1, 3, 1.0));
        assert!((result.probability - 0.5).abs() < 0.01, "expected ~0.5, got {}", result.probability);
    }

    #[test]
    fn three_slots_increase_probability_vs_one_slot() {
        let p = pool(&[("life", 100.0, true), ("res", 100.0, true), ("es", 100.0, true), ("ms", 100.0, true)]);
        let t = vec![TargetMod::new("life", true)];
        let p1 = calc_probability(&input(p.clone(), t.clone(), 1, 3, 1.0));
        let p3 = calc_probability(&input(p, t, 3, 3, 1.0));
        assert!(p3.probability > p1.probability, "3 slots must give higher probability than 1 slot");
    }

    #[test]
    fn expected_cost_equals_expected_attempts_times_cost() {
        let p = pool(&[("life", 100.0, true), ("res", 200.0, true)]);
        let t = vec![TargetMod::new("life", true)];
        let result = calc_probability(&input(p, t, 3, 3, 5.0));
        let expected = result.expected_attempts as f64 * 5.0;
        assert!((result.expected_cost - expected).abs() < 0.001);
    }

    #[test]
    fn no_targets_gives_probability_one() {
        let p = pool(&[("life", 100.0, true)]);
        let result = calc_probability(&input(p, vec![], 3, 3, 1.0));
        assert!((result.probability - 1.0).abs() < 0.001);
    }

    #[test]
    fn empty_pool_with_target_gives_zero_probability() {
        let result = calc_probability(&input(vec![], vec![TargetMod::new("life", true)], 3, 3, 1.0));
        assert_eq!(result.probability, 0.0);
    }

    #[test]
    fn prefix_and_suffix_independent() {
        // One prefix target and one suffix target — each has 50% probability independently.
        let p = pool(&[
            ("life",   100.0, true),
            ("pfx2",   100.0, true),
            ("resist", 100.0, false),
            ("sfx2",   100.0, false),
        ]);
        let t = vec![
            TargetMod::new("life",   true),
            TargetMod::new("resist", false),
        ];
        let result = calc_probability(&input(p, t, 1, 1, 1.0));
        // Each independently ~0.5 → combined ≈ 0.25
        assert!((result.probability - 0.25).abs() < 0.05,
            "expected ~0.25, got {}", result.probability);
    }

    #[test]
    fn geometric_99th_percentile_at_half_probability() {
        // P = 0.5 → 99th percentile: ceil(ln(0.01)/ln(0.5)) ≈ 7
        let n = geometric_99th_percentile(0.5);
        assert!(n >= 6 && n <= 8, "99th pct for p=0.5 should be ~7, got {n}");
    }

    #[test]
    fn geometric_99th_percentile_at_probability_one() {
        assert_eq!(geometric_99th_percentile(1.0), 1);
    }

    #[test]
    fn geometric_mean_at_half_probability() {
        assert_eq!(geometric_mean(0.5), 2); // ceil(1/0.5) = 2
    }

    #[test]
    fn tier_weight_fraction_reduces_probability() {
        let p = pool(&[("life", 100.0, true)]);
        let mut t = vec![TargetMod::new("life", true)];
        t[0].tier_weight_fraction = 0.5; // only T1 = 50% of weight
        let result = calc_probability(&input(p, t, 1, 3, 1.0));
        // P = 1.0 (only mod in pool) × 0.5 (tier fraction) = 0.5
        assert!((result.probability - 0.5).abs() < 0.01, "tier fraction should halve probability");
    }
}
