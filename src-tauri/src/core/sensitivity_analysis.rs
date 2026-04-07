/// sensitivity_analysis.rs — Numerical Partial Derivatives (Algorithm 9).
///
/// Computes the marginal DPS/life impact of adding a small delta of each stat.
/// This tells the player which stats to prioritize on new gear.
///
/// Interface: caller provides current stat values and a DPS evaluator closure.
/// The algorithm perturbs each stat by `delta` and measures the DPS change.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Named stat types that sensitivity analysis covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatType {
    FlatLife,
    PercentLife,
    FireDotMulti,
    ChaosDotMulti,
    PhysDotMulti,
    ColdDotMulti,
    FlatPhysMin,
    FlatPhysMax,
    AttackSpeed,
    CastSpeed,
    CritChance,
    CritMulti,
    GemLevel,
    FireRes,
    ColdRes,
    LightningRes,
    ChaosRes,
    Armour,
    Evasion,
    SpellDamage,
    IncreasedFireDamage,
}

/// Perturbation delta per stat type.
/// These values represent a "reasonable" single-item upgrade.
pub fn stat_delta(stat: StatType) -> f64 {
    match stat {
        StatType::FlatLife           => 50.0,
        StatType::PercentLife        => 10.0,
        StatType::FireDotMulti       => 10.0,
        StatType::ChaosDotMulti      => 10.0,
        StatType::PhysDotMulti       => 10.0,
        StatType::ColdDotMulti       => 10.0,
        StatType::FlatPhysMin        => 20.0,
        StatType::FlatPhysMax        => 20.0,
        StatType::AttackSpeed        => 10.0,
        StatType::CastSpeed          => 10.0,
        StatType::CritChance         => 10.0,
        StatType::CritMulti          => 15.0,
        StatType::GemLevel           => 1.0,
        StatType::FireRes            => 10.0,
        StatType::ColdRes            => 10.0,
        StatType::LightningRes       => 10.0,
        StatType::ChaosRes           => 10.0,
        StatType::Armour             => 1000.0,
        StatType::Evasion            => 1000.0,
        StatType::SpellDamage        => 10.0,
        StatType::IncreasedFireDamage=> 10.0,
    }
}

/// Result for one stat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatSensitivity {
    pub stat:            StatType,
    /// % DPS change for one unit of `stat_delta(stat)`.
    pub dps_change_pct:  f64,
    /// % life change for one unit of `stat_delta(stat)`.
    pub life_change_pct: f64,
    /// Diminishing returns note.
    pub returns_note:    DiminishingReturns,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiminishingReturns {
    High,
    Moderate,
    Diminishing,
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Compute sensitivity of DPS and life to each stat.
///
/// # Parameters
/// - `stats` — current build stat values keyed by StatType
/// - `eval`  — function `(stats) → (dps, life)` representing the calculator
/// - `stats_to_analyze` — which stats to perturb (defaults to all if empty)
pub fn compute_sensitivities(
    stats:            &HashMap<StatType, f64>,
    eval:             &dyn Fn(&HashMap<StatType, f64>) -> (f64, f64),
    stats_to_analyze: &[StatType],
) -> Vec<StatSensitivity> {
    let (base_dps, base_life) = eval(stats);

    let list: &[StatType] = if stats_to_analyze.is_empty() {
        &[
            StatType::FlatLife, StatType::PercentLife,
            StatType::FireDotMulti, StatType::ChaosDotMulti,
            StatType::FlatPhysMin, StatType::AttackSpeed,
            StatType::CritChance, StatType::CritMulti, StatType::GemLevel,
            StatType::SpellDamage, StatType::IncreasedFireDamage,
        ]
    } else {
        stats_to_analyze
    };

    let mut results: Vec<StatSensitivity> = list.iter().map(|&stat| {
        let delta = stat_delta(stat);
        let mut modified = stats.clone();
        *modified.entry(stat).or_insert(0.0) += delta;

        let (new_dps, new_life) = eval(&modified);
        let dps_change_pct  = if base_dps  > 0.0 { (new_dps  - base_dps)  / base_dps  * 100.0 } else { 0.0 };
        let life_change_pct = if base_life > 0.0 { (new_life - base_life) / base_life * 100.0 } else { 0.0 };

        let current = stats.get(&stat).copied().unwrap_or(0.0);
        let returns_note = if current > 300.0 {
            DiminishingReturns::Diminishing
        } else if current > 200.0 {
            DiminishingReturns::Moderate
        } else {
            DiminishingReturns::High
        };

        StatSensitivity { stat, dps_change_pct, life_change_pct, returns_note }
    }).collect();

    // Sort by DPS impact descending.
    results.sort_by(|a, b| b.dps_change_pct.partial_cmp(&a.dps_change_pct).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Find the top-N most impactful stats for DPS.
pub fn top_dps_stats(sensitivities: &[StatSensitivity], n: usize) -> &[StatSensitivity] {
    &sensitivities[..n.min(sensitivities.len())]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple evaluator: DPS = base_dps * (1 + fire_dot_multi / 100)
    ///                   life = flat_life * (1 + pct_life / 100)
    fn simple_eval(stats: &HashMap<StatType, f64>) -> (f64, f64) {
        let base_dps   = 1_000_000.0;
        let fire_multi = stats.get(&StatType::FireDotMulti).copied().unwrap_or(0.0);
        let flat_life  = stats.get(&StatType::FlatLife).copied().unwrap_or(4000.0);
        let pct_life   = stats.get(&StatType::PercentLife).copied().unwrap_or(0.0);
        let dps  = base_dps * (1.0 + fire_multi / 100.0);
        let life = flat_life * (1.0 + pct_life / 100.0);
        (dps, life)
    }

    fn stats_map(pairs: &[(StatType, f64)]) -> HashMap<StatType, f64> {
        pairs.iter().cloned().collect()
    }

    #[test]
    fn fire_dot_multi_shows_positive_dps_change() {
        let stats = stats_map(&[(StatType::FireDotMulti, 100.0)]);
        let results = compute_sensitivities(
            &stats, &simple_eval,
            &[StatType::FireDotMulti, StatType::FlatLife],
        );
        let fire = results.iter().find(|s| s.stat == StatType::FireDotMulti).unwrap();
        assert!(fire.dps_change_pct > 0.0, "fire dot multi must increase DPS");
    }

    #[test]
    fn flat_life_shows_positive_life_change() {
        let stats = stats_map(&[(StatType::FlatLife, 4000.0)]);
        let results = compute_sensitivities(
            &stats, &simple_eval,
            &[StatType::FlatLife],
        );
        let life_stat = results.iter().find(|s| s.stat == StatType::FlatLife).unwrap();
        assert!(life_stat.life_change_pct > 0.0, "flat life must increase life pool");
    }

    #[test]
    fn results_sorted_by_dps_impact_descending() {
        let stats = stats_map(&[
            (StatType::FireDotMulti, 100.0),
            (StatType::FlatLife,     4000.0),
        ]);
        let results = compute_sensitivities(
            &stats, &simple_eval,
            &[StatType::FireDotMulti, StatType::FlatLife],
        );
        if results.len() >= 2 {
            assert!(results[0].dps_change_pct >= results[1].dps_change_pct,
                "results should be sorted by DPS impact descending");
        }
    }

    #[test]
    fn stat_with_high_current_value_gets_diminishing_note() {
        let stats = stats_map(&[(StatType::FireDotMulti, 350.0)]);
        let results = compute_sensitivities(
            &stats, &simple_eval,
            &[StatType::FireDotMulti],
        );
        let fire = results.iter().find(|s| s.stat == StatType::FireDotMulti).unwrap();
        assert_eq!(fire.returns_note, DiminishingReturns::Diminishing,
            "350% existing should mark as diminishing");
    }

    #[test]
    fn stat_with_low_current_value_gets_high_returns_note() {
        let stats = stats_map(&[(StatType::FireDotMulti, 50.0)]);
        let results = compute_sensitivities(
            &stats, &simple_eval,
            &[StatType::FireDotMulti],
        );
        let fire = results.iter().find(|s| s.stat == StatType::FireDotMulti).unwrap();
        assert_eq!(fire.returns_note, DiminishingReturns::High);
    }

    #[test]
    fn empty_stats_map_still_evaluates() {
        let stats = HashMap::new();
        let results = compute_sensitivities(
            &stats, &simple_eval,
            &[StatType::FireDotMulti],
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn top_dps_stats_limits_to_n() {
        let stats = stats_map(&[(StatType::FireDotMulti, 100.0)]);
        let results = compute_sensitivities(&stats, &simple_eval, &[
            StatType::FireDotMulti, StatType::FlatLife, StatType::CritChance,
        ]);
        let top = top_dps_stats(&results, 2);
        assert!(top.len() <= 2);
    }

    #[test]
    fn top_dps_stats_does_not_panic_when_n_exceeds_len() {
        let stats = HashMap::new();
        let results = compute_sensitivities(&stats, &simple_eval, &[StatType::FireDotMulti]);
        let top = top_dps_stats(&results, 100);
        assert_eq!(top.len(), 1);
    }

    #[test]
    fn stat_delta_values_are_positive() {
        for &s in &[
            StatType::FlatLife, StatType::FireDotMulti, StatType::CritChance,
            StatType::GemLevel, StatType::Armour,
        ] {
            assert!(stat_delta(s) > 0.0, "delta for {:?} must be positive", s);
        }
    }

    #[test]
    fn zero_base_dps_returns_zero_change() {
        let eval = |_: &HashMap<StatType, f64>| -> (f64, f64) { (0.0, 1000.0) };
        let stats = HashMap::new();
        let results = compute_sensitivities(&stats, &eval, &[StatType::FireDotMulti]);
        let fire = results.iter().find(|s| s.stat == StatType::FireDotMulti).unwrap();
        assert_eq!(fire.dps_change_pct, 0.0);
    }
}
