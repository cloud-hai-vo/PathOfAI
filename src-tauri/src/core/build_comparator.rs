/// build_comparator.rs — Build comparator (Algorithm 49).
/// Tests written FIRST (TDD RED). Run `cargo test build_comparator` → all FAIL → then implement.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSnapshot {
    pub id:       String,
    pub name:     String,
    pub stats:    HashMap<String, f64>,
    pub passives: Vec<u32>,
    pub gems:     Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeltaDir { Better, Worse, Same }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatDelta {
    pub key:        String,
    pub value_a:    f64,
    pub value_b:    f64,
    pub delta:      f64,
    pub delta_pct:  f64,
    pub direction:  DeltaDir,
    pub higher_is_better: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildComparison {
    pub build_a:          String,
    pub build_b:          String,
    pub stat_deltas:      Vec<StatDelta>,
    pub tree_overlap_pct: f64,
    pub shared_gems:      Vec<String>,
    pub unique_to_a:      Vec<String>,
    pub unique_to_b:      Vec<String>,
    pub summary_winner:   Option<String>,  // id of the "better" build, or None if tie
}

// ─── Stubs → unimplemented!() → RED ──────────────────────────────────────────

/// Compare two build snapshots and return a full diff.
pub fn compare_builds(a: &BuildSnapshot, b: &BuildSnapshot) -> BuildComparison {
    // Stat deltas for all keys present in either build
    let all_keys: std::collections::HashSet<&String> =
        a.stats.keys().chain(b.stats.keys()).collect();

    let higher_is_better: &[&str] = &["dps", "life", "es", "armor", "armour",
        "evasion", "resistances", "block", "spell_block", "speed"];

    let stat_deltas: Vec<StatDelta> = all_keys.into_iter().map(|k| {
        let va = *a.stats.get(k).unwrap_or(&0.0);
        let vb = *b.stats.get(k).unwrap_or(&0.0);
        let hib = higher_is_better.iter().any(|&h| k.contains(h));
        stat_delta(k, va, vb, hib)
    }).collect();

    let tree_overlap_pct = tree_overlap(&a.passives, &b.passives);

    let set_a: std::collections::HashSet<&String> = a.gems.iter().collect();
    let set_b: std::collections::HashSet<&String> = b.gems.iter().collect();
    let shared_gems: Vec<String> = set_a.intersection(&set_b).map(|s| s.to_string()).collect();
    let unique_to_a: Vec<String> = set_a.difference(&set_b).map(|s| s.to_string()).collect();
    let unique_to_b: Vec<String> = set_b.difference(&set_a).map(|s| s.to_string()).collect();

    let summary_winner = winner_from_deltas(&stat_deltas, &a.id, &b.id);

    BuildComparison {
        build_a: a.id.clone(), build_b: b.id.clone(),
        stat_deltas, tree_overlap_pct, shared_gems, unique_to_a, unique_to_b,
        summary_winner,
    }
}

pub fn stat_delta(key: &str, val_a: f64, val_b: f64, higher_is_better: bool) -> StatDelta {
    let delta = val_b - val_a;
    let delta_pct = if val_a != 0.0 { delta / val_a.abs() * 100.0 } else { 0.0 };

    let direction = if delta == 0.0 {
        DeltaDir::Same
    } else if higher_is_better {
        if delta > 0.0 { DeltaDir::Better } else { DeltaDir::Worse }
    } else {
        if delta < 0.0 { DeltaDir::Better } else { DeltaDir::Worse }
    };

    StatDelta { key: key.to_string(), value_a: val_a, value_b: val_b, delta, delta_pct, direction, higher_is_better }
}

pub fn tree_overlap(a: &[u32], b: &[u32]) -> f64 {
    if a.is_empty() && b.is_empty() { return 0.0; }
    let set_a: std::collections::HashSet<u32> = a.iter().cloned().collect();
    let set_b: std::collections::HashSet<u32> = b.iter().cloned().collect();
    let shared = set_a.intersection(&set_b).count();
    let union  = set_a.union(&set_b).count();
    if union == 0 { return 0.0; }
    shared as f64 / union as f64 * 100.0
}

fn winner_from_deltas(deltas: &[StatDelta], id_a: &str, id_b: &str) -> Option<String> {
    let better = deltas.iter().filter(|d| d.direction == DeltaDir::Better).count();
    let worse  = deltas.iter().filter(|d| d.direction == DeltaDir::Worse).count();
    if better > worse { Some(id_b.to_string()) }
    else if worse > better { Some(id_a.to_string()) }
    else { None }
}

pub fn winner(comparison: &BuildComparison) -> Option<String> {
    winner_from_deltas(&comparison.stat_deltas, &comparison.build_a, &comparison.build_b)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn build(id: &str, stats: &[(&str, f64)], passives: Vec<u32>, gems: Vec<&str>) -> BuildSnapshot {
        BuildSnapshot {
            id: id.to_string(), name: id.to_string(),
            stats: stats.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            passives,
            gems: gems.into_iter().map(|g| g.to_string()).collect(),
        }
    }

    // ── stat_delta ────────────────────────────────────────────────────────────

    #[test]
    fn stat_delta_better_when_higher_is_better_and_b_gt_a() {
        let d = stat_delta("dps", 1000.0, 1500.0, true);
        assert_eq!(d.direction, DeltaDir::Better);
        assert!((d.delta - 500.0).abs() < 0.001);
    }

    #[test]
    fn stat_delta_worse_when_higher_is_better_and_b_lt_a() {
        let d = stat_delta("dps", 1500.0, 1000.0, true);
        assert_eq!(d.direction, DeltaDir::Worse);
        assert!((d.delta - (-500.0)).abs() < 0.001);
    }

    #[test]
    fn stat_delta_better_when_lower_is_better_and_b_lt_a() {
        // e.g. damage taken — lower is better
        let d = stat_delta("dmg_taken", 1000.0, 800.0, false);
        assert_eq!(d.direction, DeltaDir::Better);
    }

    #[test]
    fn stat_delta_same_when_equal() {
        let d = stat_delta("life", 5000.0, 5000.0, true);
        assert_eq!(d.direction, DeltaDir::Same);
        assert_eq!(d.delta, 0.0);
        assert_eq!(d.delta_pct, 0.0);
    }

    #[test]
    fn stat_delta_pct_formula_correct() {
        // (1500 - 1000) / |1000| * 100 = 50%
        let d = stat_delta("dps", 1000.0, 1500.0, true);
        assert!((d.delta_pct - 50.0).abs() < 0.001, "got {}", d.delta_pct);
    }

    #[test]
    fn stat_delta_pct_zero_baseline() {
        // when a == 0, delta_pct is undefined; should not panic
        let d = stat_delta("x", 0.0, 100.0, true);
        assert!(!d.delta_pct.is_nan() || d.delta_pct.is_infinite() || d.delta_pct == 0.0);
    }

    // ── tree_overlap ──────────────────────────────────────────────────────────

    #[test]
    fn tree_overlap_identical_is_100() {
        assert!((tree_overlap(&[1, 2, 3], &[1, 2, 3]) - 100.0).abs() < 0.001);
    }

    #[test]
    fn tree_overlap_disjoint_is_zero() {
        assert_eq!(tree_overlap(&[1, 2], &[3, 4]), 0.0);
    }

    #[test]
    fn tree_overlap_empty_is_zero() {
        assert_eq!(tree_overlap(&[], &[]), 0.0);
    }

    #[test]
    fn tree_overlap_partial_correct() {
        let pct = tree_overlap(&[1, 2, 3, 4], &[3, 4, 5, 6]);
        assert!((pct - 33.333).abs() < 0.1, "got {pct}");
    }

    // ── compare_builds ────────────────────────────────────────────────────────

    #[test]
    fn compare_produces_stat_delta_per_shared_key() {
        let a = build("A", &[("dps", 1000.0), ("life", 4000.0)], vec![], vec![]);
        let b = build("B", &[("dps", 1500.0), ("life", 3500.0)], vec![], vec![]);
        let cmp = compare_builds(&a, &b);
        assert_eq!(cmp.stat_deltas.len(), 2, "one delta per shared stat key");
    }

    #[test]
    fn compare_tree_overlap_uses_passives() {
        let a = build("A", &[], vec![1, 2, 3], vec![]);
        let b = build("B", &[], vec![1, 2, 4], vec![]);
        let cmp = compare_builds(&a, &b);
        // shared={1,2}=2, union={1,2,3,4}=4 → 50%
        assert!((cmp.tree_overlap_pct - 50.0).abs() < 0.1, "got {}", cmp.tree_overlap_pct);
    }

    #[test]
    fn compare_gems_split_correctly() {
        let a = build("A", &[], vec![], vec!["Fireball", "Arc"]);
        let b = build("B", &[], vec![], vec!["Arc", "Frostbolt"]);
        let cmp = compare_builds(&a, &b);
        assert!(cmp.shared_gems.contains(&"Arc".to_string()));
        assert!(cmp.unique_to_a.contains(&"Fireball".to_string()));
        assert!(cmp.unique_to_b.contains(&"Frostbolt".to_string()));
    }

    #[test]
    fn compare_build_ids_are_set_correctly() {
        let a = build("MyBuild", &[], vec![], vec![]);
        let b = build("OtherBuild", &[], vec![], vec![]);
        let cmp = compare_builds(&a, &b);
        assert_eq!(cmp.build_a, "MyBuild");
        assert_eq!(cmp.build_b, "OtherBuild");
    }

    // ── winner ────────────────────────────────────────────────────────────────

    #[test]
    fn winner_is_b_when_all_stats_better_in_b() {
        let a = build("A", &[("dps", 1000.0)], vec![], vec![]);
        let b = build("B", &[("dps", 2000.0)], vec![], vec![]);
        let cmp = compare_builds(&a, &b);
        let w = winner(&cmp);
        assert_eq!(w.as_deref(), Some("B"));
    }

    #[test]
    fn winner_is_none_on_tie() {
        // same stats → no winner
        let a = build("A", &[("dps", 1000.0)], vec![], vec![]);
        let b = build("B", &[("dps", 1000.0)], vec![], vec![]);
        let cmp = compare_builds(&a, &b);
        assert!(winner(&cmp).is_none());
    }
}
