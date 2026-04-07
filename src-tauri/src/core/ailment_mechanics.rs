/// ailment_mechanics.rs — Ailment Mechanics (Algorithm 26).
///
/// Computes DPS and durations for PoE's six ailments:
/// Ignite, Chill, Freeze, Shock, Poison, Bleed.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgniteResult {
    pub dps_per_second:   f64,
    pub duration_secs:    f64,
    pub total_damage:     f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChillResult {
    pub effect_pct:       f64,  // 5–30% action speed reduction
    pub duration_secs:    f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeResult {
    pub can_freeze:       bool,
    pub duration_secs:    f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShockResult {
    pub effect_pct:       f64,  // 1–50% increased damage taken
    pub duration_secs:    f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoisonResult {
    pub dps_per_stack:    f64,
    pub max_stacks:       f64,
    pub total_dps:        f64,
    pub duration_secs:    f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BleedResult {
    pub dps_stationary:   f64,
    pub dps_moving:       f64,
    pub active_dps:       f64,
    pub duration_secs:    f64,
    pub active_stacks:    u32,
}

// ─── Ignite ───────────────────────────────────────────────────────────────────

/// Calculate ignite DPS and duration.
/// `fire_hit` — fire damage of the igniting hit
/// `fire_dot_multi_pct` — e.g. 30 for "30% increased fire DoT multi"
/// `increased_burning_pct` — e.g. 20 for "20% increased burning damage"
/// `increased_duration_pct` — e.g. 10 for "10% increased ailment duration"
pub fn calc_ignite(
    fire_hit:               f64,
    fire_dot_multi_pct:     f64,
    increased_burning_pct:  f64,
    increased_duration_pct: f64,
) -> IgniteResult {
    let duration = 4.0 * (1.0 + increased_duration_pct / 100.0);
    // Ignite deals the hit's fire damage over 4s base (= 12.5%/s of hit)
    let dps = fire_hit
        * (0.5 / duration)          // distribute hit over duration
        * (1.0 + fire_dot_multi_pct / 100.0)
        * (1.0 + increased_burning_pct / 100.0);
    IgniteResult {
        dps_per_second: dps,
        duration_secs:  duration,
        total_damage:   dps * duration,
    }
}

// ─── Chill ────────────────────────────────────────────────────────────────────

/// Calculate chill effect and duration.
/// `cold_hit` — cold damage of the chilling hit
/// `target_max_life` — enemy maximum life
pub fn calc_chill(
    cold_hit:               f64,
    target_max_life:        f64,
    increased_effect_pct:   f64,
    increased_duration_pct: f64,
) -> ChillResult {
    let raw_pct = (cold_hit / target_max_life.max(1.0) * 100.0).sqrt() * 10.0;
    let effect = (raw_pct * (1.0 + increased_effect_pct / 100.0)).clamp(5.0, 30.0);
    let duration = 2.0 * (1.0 + increased_duration_pct / 100.0);
    ChillResult { effect_pct: effect, duration_secs: duration }
}

// ─── Freeze ───────────────────────────────────────────────────────────────────

/// Calculate freeze ability and duration.
/// `cold_hit` — cold damage of the freezing hit
/// `target_max_life` — enemy maximum life
pub fn calc_freeze(
    cold_hit:           f64,
    target_max_life:    f64,
) -> FreezeResult {
    let threshold = target_max_life * 0.0015; // 0.15% of max life
    let can_freeze = cold_hit >= threshold;

    let duration = if can_freeze {
        let raw = (cold_hit / threshold.max(0.001) - 1.0) * 0.06; // base freeze ≈ 0.06s scale
        raw.clamp(0.3, 60.0)
    } else {
        0.0
    };

    FreezeResult { can_freeze, duration_secs: duration }
}

// ─── Shock ────────────────────────────────────────────────────────────────────

/// Calculate shock effect and duration.
/// `lightning_hit` — lightning damage of the shocking hit
/// `target_max_life` — enemy maximum life
/// `has_always_shocks` — e.g. Shaper of Storms (minimum 15% shock)
pub fn calc_shock(
    lightning_hit:          f64,
    target_max_life:        f64,
    increased_effect_pct:   f64,
    increased_duration_pct: f64,
    has_always_shocks:      bool,
) -> ShockResult {
    let raw_pct = (lightning_hit / target_max_life.max(1.0) * 100.0).sqrt() * 10.0;
    let mut effect = (raw_pct * (1.0 + increased_effect_pct / 100.0)).clamp(1.0, 50.0);
    if has_always_shocks {
        effect = effect.max(15.0);
    }
    let duration = 2.0 * (1.0 + increased_duration_pct / 100.0);
    ShockResult { effect_pct: effect, duration_secs: duration }
}

// ─── Poison ───────────────────────────────────────────────────────────────────

/// Calculate poison DPS and stack count.
/// `phys_chaos_hit` — physical + chaos damage of the hit
/// `hit_rate` — attacks or casts per second
/// `poison_chance_pct` — 0-100
/// `chaos_dot_multi_pct` — e.g. 30 for "30% increased chaos DoT multi"
/// `increased_poison_pct` — e.g. 20 for "20% increased poison damage"
/// `increased_duration_pct` — e.g. 0
pub fn calc_poison(
    phys_chaos_hit:         f64,
    hit_rate:               f64,
    poison_chance_pct:      f64,
    chaos_dot_multi_pct:    f64,
    increased_poison_pct:   f64,
    increased_duration_pct: f64,
) -> PoisonResult {
    let duration = 2.0 * (1.0 + increased_duration_pct / 100.0);
    let dps_per_stack = phys_chaos_hit * 0.10
        * (1.0 + chaos_dot_multi_pct / 100.0)
        * (1.0 + increased_poison_pct / 100.0);

    // Sustainable max stacks = hit_rate × chance × duration
    let chance = poison_chance_pct / 100.0;
    let max_stacks = (hit_rate * chance * duration).max(0.0);
    let total_dps = max_stacks * dps_per_stack;

    PoisonResult {
        dps_per_stack,
        max_stacks,
        total_dps,
        duration_secs: duration,
    }
}

// ─── Bleed ────────────────────────────────────────────────────────────────────

/// Calculate bleed DPS.
/// `phys_hit` — physical damage of the hit
/// `hit_rate` — attacks per second
/// `bleed_chance_pct` — 0-100
/// `phys_dot_multi_pct` — physical DoT multiplier %
/// `increased_bleed_pct` — increased bleed damage %
/// `increased_duration_pct` — increased bleed duration %
/// `has_crimson_dance` — allows 8 simultaneous stacks
/// `target_is_moving` — 3× DPS when target is moving
pub fn calc_bleed(
    phys_hit:               f64,
    hit_rate:               f64,
    bleed_chance_pct:       f64,
    phys_dot_multi_pct:     f64,
    increased_bleed_pct:    f64,
    increased_duration_pct: f64,
    has_crimson_dance:      bool,
    target_is_moving:       bool,
) -> BleedResult {
    let duration = 5.0 * (1.0 + increased_duration_pct / 100.0);

    let base_dps_stat = phys_hit * 0.14  // 70% over 5s → 14%/s
        * (1.0 + phys_dot_multi_pct / 100.0)
        * (1.0 + increased_bleed_pct / 100.0);

    let dps_stationary = base_dps_stat;
    let dps_moving     = base_dps_stat * 3.0; // 3× when target moves

    let max_stacks = if has_crimson_dance { 8u32 } else { 1u32 };
    let chance = bleed_chance_pct / 100.0;
    let estimated_stacks = (hit_rate * chance * duration).min(max_stacks as f64) as u32;
    let active_stacks = estimated_stacks.max(if chance > 0.0 { 1 } else { 0 }).min(max_stacks);

    let per_stack_dps = if target_is_moving { dps_moving } else { dps_stationary };
    let active_dps = per_stack_dps * active_stacks as f64;

    BleedResult {
        dps_stationary,
        dps_moving,
        active_dps,
        duration_secs: duration,
        active_stacks,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Ignite ────────────────────────────────────────────────────────────────

    #[test]
    fn ignite_base_dps_is_12_5_pct_of_hit_per_second() {
        // Base: 4s duration, no mods → 12.5%/s of the hit
        let r = calc_ignite(1000.0, 0.0, 0.0, 0.0);
        assert!((r.dps_per_second - 125.0).abs() < 0.1,
            "expected 125 dps (12.5% of 1000), got {}", r.dps_per_second);
    }

    #[test]
    fn ignite_fire_dot_multi_scales_dps() {
        let base = calc_ignite(1000.0, 0.0, 0.0, 0.0);
        let boosted = calc_ignite(1000.0, 50.0, 0.0, 0.0); // +50% DoT multi
        assert!((boosted.dps_per_second / base.dps_per_second - 1.5).abs() < 0.01,
            "50% DoT multi should give 1.5× DPS");
    }

    #[test]
    fn ignite_total_damage_equals_dps_times_duration() {
        let r = calc_ignite(1000.0, 20.0, 10.0, 0.0);
        assert!((r.total_damage - r.dps_per_second * r.duration_secs).abs() < 0.01);
    }

    #[test]
    fn ignite_duration_scales_with_increased_duration() {
        let base = calc_ignite(1000.0, 0.0, 0.0, 0.0);
        let longer = calc_ignite(1000.0, 0.0, 0.0, 50.0); // +50% duration
        assert!((longer.duration_secs - base.duration_secs * 1.5).abs() < 0.01);
    }

    // ── Chill ─────────────────────────────────────────────────────────────────

    #[test]
    fn chill_effect_scales_with_hit_proportion() {
        // 10% of max life hit → sqrt(10) * 10 ≈ 31.6% → capped to 30%
        let r = calc_chill(1000.0, 10_000.0, 0.0, 0.0);
        assert!((r.effect_pct - 30.0).abs() < 1.0, "large hit should cap chill at 30%");
    }

    #[test]
    fn chill_effect_minimum_5_pct() {
        // Very small hit → still at least 5%
        let r = calc_chill(1.0, 1_000_000.0, 0.0, 0.0);
        assert!(r.effect_pct >= 5.0, "chill must be at least 5%");
    }

    #[test]
    fn chill_effect_capped_at_30_pct() {
        let r = calc_chill(100_000.0, 100.0, 0.0, 0.0); // massive hit
        assert!(r.effect_pct <= 30.0, "chill must not exceed 30%");
    }

    #[test]
    fn chill_base_duration_is_2s() {
        let r = calc_chill(100.0, 1000.0, 0.0, 0.0);
        assert!((r.duration_secs - 2.0).abs() < 0.01);
    }

    // ── Freeze ────────────────────────────────────────────────────────────────

    #[test]
    fn freeze_requires_hit_above_threshold() {
        // 0.15% of 100k = 150 threshold
        let r_below = calc_freeze(100.0, 100_000.0);
        let r_above = calc_freeze(300.0, 100_000.0);
        assert!(!r_below.can_freeze, "small hit should not freeze");
        assert!(r_above.can_freeze, "hit above threshold should freeze");
    }

    #[test]
    fn freeze_duration_zero_when_cannot_freeze() {
        let r = calc_freeze(1.0, 1_000_000.0);
        assert_eq!(r.duration_secs, 0.0);
    }

    #[test]
    fn freeze_duration_at_least_0_3s() {
        let r = calc_freeze(1_000_000.0, 1.0); // huge hit
        assert!(r.duration_secs <= 60.0, "freeze capped at 60s");
        assert!(r.duration_secs >= 0.3 || !r.can_freeze);
    }

    // ── Shock ─────────────────────────────────────────────────────────────────

    #[test]
    fn shock_effect_small_hit_is_at_least_1_pct() {
        let r = calc_shock(1.0, 1_000_000.0, 0.0, 0.0, false);
        assert!(r.effect_pct >= 1.0);
    }

    #[test]
    fn shock_capped_at_50_pct() {
        let r = calc_shock(1_000_000.0, 100.0, 0.0, 0.0, false);
        assert!(r.effect_pct <= 50.0);
    }

    #[test]
    fn always_shocks_gives_minimum_15_pct() {
        let r = calc_shock(1.0, 1_000_000.0, 0.0, 0.0, true);
        assert!(r.effect_pct >= 15.0, "always shocks should give at least 15% shock");
    }

    #[test]
    fn shock_base_duration_is_2s() {
        let r = calc_shock(1000.0, 10_000.0, 0.0, 0.0, false);
        assert!((r.duration_secs - 2.0).abs() < 0.01);
    }

    // ── Poison ────────────────────────────────────────────────────────────────

    #[test]
    fn poison_base_dps_is_10_pct_of_hit() {
        let r = calc_poison(1000.0, 1.0, 100.0, 0.0, 0.0, 0.0);
        assert!((r.dps_per_stack - 100.0).abs() < 0.1,
            "10% of 1000 = 100 dps per stack, got {}", r.dps_per_stack);
    }

    #[test]
    fn poison_stacks_scale_with_hit_rate_and_duration() {
        // 5 hits/s × 100% chance × 2s duration = 10 stacks
        let r = calc_poison(100.0, 5.0, 100.0, 0.0, 0.0, 0.0);
        assert!((r.max_stacks - 10.0).abs() < 0.1, "expected 10 stacks, got {}", r.max_stacks);
    }

    #[test]
    fn poison_total_dps_is_stacks_times_per_stack() {
        let r = calc_poison(1000.0, 2.0, 50.0, 0.0, 0.0, 0.0);
        assert!((r.total_dps - r.max_stacks * r.dps_per_stack).abs() < 0.01);
    }

    // ── Bleed ─────────────────────────────────────────────────────────────────

    #[test]
    fn bleed_base_stationary_dps_is_14_pct_per_second() {
        let r = calc_bleed(1000.0, 1.0, 100.0, 0.0, 0.0, 0.0, false, false);
        assert!((r.dps_stationary - 140.0).abs() < 0.1,
            "14% of 1000 = 140 dps, got {}", r.dps_stationary);
    }

    #[test]
    fn bleed_moving_is_3x_stationary() {
        let r = calc_bleed(1000.0, 1.0, 100.0, 0.0, 0.0, 0.0, false, true);
        let stat = calc_bleed(1000.0, 1.0, 100.0, 0.0, 0.0, 0.0, false, false);
        assert!((r.dps_moving - stat.dps_stationary * 3.0).abs() < 0.1);
    }

    #[test]
    fn bleed_without_crimson_dance_max_1_stack() {
        let r = calc_bleed(1000.0, 10.0, 100.0, 0.0, 0.0, 0.0, false, false);
        assert_eq!(r.active_stacks, 1, "without crimson dance only 1 stack active");
    }

    #[test]
    fn bleed_crimson_dance_allows_up_to_8_stacks() {
        let r = calc_bleed(1000.0, 10.0, 100.0, 0.0, 0.0, 0.0, true, false);
        assert!(r.active_stacks <= 8);
        assert!(r.active_stacks > 1, "crimson dance should allow multiple stacks");
    }

    #[test]
    fn bleed_no_chance_means_no_bleed() {
        let r = calc_bleed(1000.0, 10.0, 0.0, 0.0, 0.0, 0.0, false, false);
        assert_eq!(r.active_stacks, 0);
        assert_eq!(r.active_dps, 0.0);
    }

    #[test]
    fn bleed_phys_dot_multi_scales_dps() {
        let base = calc_bleed(1000.0, 1.0, 100.0, 0.0, 0.0, 0.0, false, false);
        let boosted = calc_bleed(1000.0, 1.0, 100.0, 100.0, 0.0, 0.0, false, false);
        assert!((boosted.dps_stationary - base.dps_stationary * 2.0).abs() < 0.1,
            "100% phys dot multi should double dps");
    }
}
