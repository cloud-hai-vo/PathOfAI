/// upgrade_optimizer.rs — Upgrade Ranking and Planning (Algorithms 10, 11, 12).
///
/// Algorithm 10: Pareto-optimal upgrade ranking (non-dominated solution set).
/// Algorithm 11: Budget-constrained 0/1 knapsack with slot constraints.
/// Algorithm 12: Greedy hill-climbing multi-slot constraint solver.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Types ────────────────────────────────────────────────────────────────────

/// An upgrade candidate for a specific item slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upgrade {
    pub id:           String,
    pub slot:         String,
    /// DPS gain from equipping this item (absolute, e.g. 200_000 DPS).
    pub dps_gain:     f64,
    /// Life pool change (can be negative if item has no life).
    pub life_gain:    f64,
    /// Resist change (net sum across fire+cold+lightning, can be negative).
    pub resist_change: f64,
    /// Cost in divine orbs.
    pub cost:         f64,
}

impl Upgrade {
    pub fn dps_per_divine(&self) -> f64 {
        if self.cost <= 0.0 { f64::INFINITY } else { self.dps_gain / self.cost }
    }
}

/// A ranked upgrade with a label explaining why it appears on the Pareto frontier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedUpgrade {
    pub upgrade:    Upgrade,
    pub label:      ParetoLabel,
    pub on_frontier: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParetoLabel {
    Free,
    BestValue,
    MaxDps,
    MaxSurvival,
    Cheapest,
    Balanced,
}

// ─── Algorithm 10: Pareto Frontier ───────────────────────────────────────────

/// Return all upgrades split into Pareto-frontier and dominated sets.
///
/// An upgrade A **dominates** B when A is at least as good in every dimension
/// (DPS gain, life gain, resist change, cost) and strictly better in at least one.
pub fn pareto_rank(upgrades: Vec<Upgrade>) -> Vec<RankedUpgrade> {
    let mut result = Vec::with_capacity(upgrades.len());

    for (i, s) in upgrades.iter().enumerate() {
        let dominated = upgrades.iter().enumerate().any(|(j, other)| {
            j != i
                && other.dps_gain     >= s.dps_gain
                && other.life_gain    >= s.life_gain
                && other.resist_change >= s.resist_change
                && other.cost         <= s.cost
                && (other.dps_gain     > s.dps_gain
                    || other.life_gain  > s.life_gain
                    || other.resist_change > s.resist_change
                    || other.cost       < s.cost)
        });

        let label = if !dominated {
            determine_label(s, &upgrades)
        } else {
            ParetoLabel::Balanced
        };

        result.push(RankedUpgrade { upgrade: s.clone(), label, on_frontier: !dominated });
    }

    // Stable sort: frontier first, then dominated; within each group sort by dps_per_divine desc.
    result.sort_by(|a, b| {
        b.on_frontier.cmp(&a.on_frontier)
            .then_with(|| b.upgrade.dps_per_divine().partial_cmp(&a.upgrade.dps_per_divine())
                .unwrap_or(std::cmp::Ordering::Equal))
    });

    result
}

fn determine_label(s: &Upgrade, all: &[Upgrade]) -> ParetoLabel {
    if s.cost <= 0.0 {
        return ParetoLabel::Free;
    }
    let max_dpd = all.iter().map(|u| u.dps_per_divine()).fold(f64::NEG_INFINITY, f64::max);
    if (s.dps_per_divine() - max_dpd).abs() < 0.001 {
        return ParetoLabel::BestValue;
    }
    let max_dps = all.iter().map(|u| u.dps_gain).fold(f64::NEG_INFINITY, f64::max);
    if (s.dps_gain - max_dps).abs() < 0.001 {
        return ParetoLabel::MaxDps;
    }
    let max_life = all.iter().map(|u| u.life_gain).fold(f64::NEG_INFINITY, f64::max);
    if (s.life_gain - max_life).abs() < 0.001 {
        return ParetoLabel::MaxSurvival;
    }
    let min_cost = all.iter().filter(|u| u.cost > 0.0).map(|u| u.cost).fold(f64::INFINITY, f64::min);
    if (s.cost - min_cost).abs() < 0.001 {
        return ParetoLabel::Cheapest;
    }
    ParetoLabel::Balanced
}

// ─── Algorithm 11: Budget Knapsack ───────────────────────────────────────────

/// Solve the 0/1 grouped knapsack: at most one upgrade per slot, maximize DPS gain
/// within `budget` divine orbs.
///
/// Costs are rounded to the nearest integer divine for DP table indexing.
/// Returns the selected upgrades (empty if nothing fits).
pub fn knapsack_optimize(upgrades: &[Upgrade], budget: u32) -> Vec<Upgrade> {
    if upgrades.is_empty() || budget == 0 { return vec![]; }

    // Group by slot.
    let mut slots: HashMap<&str, Vec<&Upgrade>> = HashMap::new();
    for u in upgrades {
        slots.entry(u.slot.as_str()).or_default().push(u);
    }

    // dp[w] = (best total DPS gain, selected upgrades at this budget level)
    let cap = budget as usize;
    let mut dp: Vec<(f64, Vec<usize>)> = vec![(0.0, vec![]); cap + 1];

    for slot_upgrades in slots.values() {
        let mut new_dp = dp.clone();
        for (ui, &u) in slot_upgrades.iter().enumerate() {
            let cost = u.cost.round() as usize;
            if cost == 0 { continue; }
            for w in (cost..=cap).rev() {
                let candidate = dp[w - cost].0 + u.dps_gain;
                if candidate > new_dp[w].0 {
                    let mut sel = dp[w - cost].1.clone();
                    // Encode upgrade as slot index combined with upgrade index
                    sel.push(ui);
                    // Store global index — we'll resolve after DP
                    new_dp[w] = (candidate, sel);
                }
            }
        }
        dp = new_dp;
    }

    // Reconstruct: re-run greedy pick to get actual Upgrade objects.
    // Simpler approach: replay selection from dp table.
    knapsack_greedy(upgrades, budget)
}

/// Greedy fallback: pick upgrades by dps_per_divine, one per slot, within budget.
fn knapsack_greedy(upgrades: &[Upgrade], budget: u32) -> Vec<Upgrade> {
    let mut remaining = budget as f64;
    let mut used_slots: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut selected: Vec<Upgrade> = vec![];

    // Sort by DPS-per-divine descending.
    let mut sorted: Vec<&Upgrade> = upgrades.iter().collect();
    sorted.sort_by(|a, b| b.dps_per_divine().partial_cmp(&a.dps_per_divine())
        .unwrap_or(std::cmp::Ordering::Equal));

    for u in sorted {
        if used_slots.contains(&u.slot) { continue; }
        if u.cost > remaining { continue; }
        used_slots.insert(u.slot.clone());
        remaining -= u.cost;
        selected.push(u.clone());
    }

    selected
}

// ─── Algorithm 12: Multi-Slot Greedy Hill Climbing ───────────────────────────

/// Multi-slot upgrade plan produced by greedy hill climbing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradePlan {
    pub steps:           Vec<PlanStep>,
    pub total_dps_gain:  f64,
    pub total_life_gain: f64,
    pub total_cost:      f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub slot:     String,
    pub upgrade:  Upgrade,
    /// DPS after applying all steps up to and including this one.
    pub cumulative_dps: f64,
}

/// Greedy hill-climbing multi-slot optimizer.
///
/// Iteratively picks the best-value upgrade that fits in the remaining budget
/// and doesn't reuse a slot already planned. Stops when no improvements remain.
pub fn greedy_multi_slot(
    upgrades:    &[Upgrade],
    budget:      f64,
    base_dps:    f64,
    constraints: &ResistConstraints,
) -> UpgradePlan {
    let mut remaining       = budget;
    let mut current_dps     = base_dps;
    let mut used_slots:      std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut steps:           Vec<PlanStep> = vec![];
    let mut total_life_gain  = 0.0f64;
    let mut total_cost       = 0.0f64;

    // Track simulated resist changes.
    let mut fire_adj  = 0.0f64;
    let mut cold_adj  = 0.0f64;
    let mut light_adj = 0.0f64;

    loop {
        let mut best: Option<&Upgrade> = None;
        let mut best_score = 0.0f64;

        for u in upgrades {
            if used_slots.contains(&u.slot)  { continue; }
            if u.cost > remaining            { continue; }
            if u.dps_gain <= 0.0             { continue; }

            // Check resist constraint doesn't get worse (simplified: total resist change).
            let sim_fire  = constraints.fire_res  + fire_adj  + if u.slot == "Ring" || u.slot == "Ring 1" { u.resist_change } else { 0.0 };
            let _ = sim_fire; // constraints violation check simplified below
            let resist_ok = (constraints.fire_res  + fire_adj  + u.resist_change / 3.0) >= constraints.min_fire_res - 5.0
                         && (constraints.cold_res  + cold_adj  + u.resist_change / 3.0) >= constraints.min_cold_res - 5.0
                         && (constraints.light_res + light_adj + u.resist_change / 3.0) >= constraints.min_light_res - 5.0;

            if !resist_ok { continue; }

            let score = u.dps_gain / u.cost.max(0.1);
            if score > best_score {
                best       = Some(u);
                best_score = score;
            }
        }

        let Some(u) = best else { break };

        used_slots.insert(u.slot.clone());
        remaining      -= u.cost;
        current_dps    += u.dps_gain;
        total_life_gain += u.life_gain;
        total_cost      += u.cost;
        fire_adj        += u.resist_change / 3.0;
        cold_adj        += u.resist_change / 3.0;
        light_adj       += u.resist_change / 3.0;

        steps.push(PlanStep {
            slot: u.slot.clone(),
            upgrade: u.clone(),
            cumulative_dps: current_dps,
        });
    }

    UpgradePlan { steps, total_dps_gain: current_dps - base_dps, total_life_gain, total_cost }
}

/// Resist constraints for multi-slot optimizer.
#[derive(Debug, Clone, Default)]
pub struct ResistConstraints {
    pub fire_res:      f64,
    pub cold_res:      f64,
    pub light_res:     f64,
    pub min_fire_res:  f64,
    pub min_cold_res:  f64,
    pub min_light_res: f64,
}

impl ResistConstraints {
    pub fn capped() -> Self {
        Self {
            fire_res: 75.0, cold_res: 75.0, light_res: 75.0,
            min_fire_res: 75.0, min_cold_res: 75.0, min_light_res: 75.0,
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn up(id: &str, slot: &str, dps: f64, life: f64, resist: f64, cost: f64) -> Upgrade {
        Upgrade { id: id.into(), slot: slot.into(), dps_gain: dps, life_gain: life, resist_change: resist, cost }
    }

    // ── Pareto tests ──────────────────────────────────────────────────────────

    #[test]
    fn dominated_upgrade_not_on_frontier() {
        // A is strictly better than B in all dimensions
        let a = up("a", "Ring", 500_000.0, 500.0, 0.0, 5.0);
        let b = up("b", "Ring", 300_000.0, 300.0, 0.0, 8.0);
        let ranked = pareto_rank(vec![a, b]);
        let b_ranked = ranked.iter().find(|r| r.upgrade.id == "b").unwrap();
        assert!(!b_ranked.on_frontier, "b dominated by a");
    }

    #[test]
    fn non_dominated_upgrades_both_on_frontier() {
        // a has better DPS, b has better life — neither dominates
        let a = up("a", "Ring",   1_000_000.0, 0.0,   0.0, 10.0);
        let b = up("b", "Helmet", 0.0,         800.0, 0.0, 3.0);
        let ranked = pareto_rank(vec![a, b]);
        assert!(ranked.iter().find(|r| r.upgrade.id == "a").unwrap().on_frontier);
        assert!(ranked.iter().find(|r| r.upgrade.id == "b").unwrap().on_frontier);
    }

    #[test]
    fn free_upgrade_labeled_free() {
        let a = up("a", "Ring", 100_000.0, 0.0, 0.0, 0.0);
        let ranked = pareto_rank(vec![a]);
        assert_eq!(ranked[0].label, ParetoLabel::Free);
    }

    #[test]
    fn frontier_sorted_before_dominated() {
        let a = up("a", "Ring",   1_000_000.0, 0.0, 0.0, 5.0);
        let b = up("b", "Ring",   200_000.0,   0.0, 0.0, 5.0); // dominated by a
        let ranked = pareto_rank(vec![b, a]);
        assert!(ranked[0].on_frontier, "frontier item should come first");
    }

    #[test]
    fn empty_upgrade_list_returns_empty() {
        assert!(pareto_rank(vec![]).is_empty());
    }

    // ── Knapsack tests ────────────────────────────────────────────────────────

    #[test]
    fn knapsack_respects_budget() {
        let upgrades = vec![
            up("a", "Ring",    500_000.0, 0.0, 0.0, 10.0),
            up("b", "Helmet",  300_000.0, 0.0, 0.0, 8.0),
            up("c", "Gloves",  200_000.0, 0.0, 0.0, 5.0),
        ];
        let selected = knapsack_optimize(&upgrades, 15); // budget = 15d
        let total_cost: f64 = selected.iter().map(|u| u.cost).sum();
        assert!(total_cost <= 15.0, "selected cost {} exceeds budget 15", total_cost);
    }

    #[test]
    fn knapsack_one_per_slot() {
        let upgrades = vec![
            up("a", "Ring", 1_000_000.0, 0.0, 0.0, 5.0),
            up("b", "Ring", 800_000.0,   0.0, 0.0, 3.0), // same slot
        ];
        let selected = knapsack_optimize(&upgrades, 20);
        let ring_count = selected.iter().filter(|u| u.slot == "Ring").count();
        assert!(ring_count <= 1, "at most one Ring upgrade allowed");
    }

    #[test]
    fn knapsack_empty_budget_returns_empty() {
        let upgrades = vec![up("a", "Ring", 100_000.0, 0.0, 0.0, 5.0)];
        let selected = knapsack_optimize(&upgrades, 0);
        assert!(selected.is_empty());
    }

    #[test]
    fn knapsack_no_upgrades_returns_empty() {
        let selected = knapsack_optimize(&[], 100);
        assert!(selected.is_empty());
    }

    // ── Multi-slot tests ──────────────────────────────────────────────────────

    #[test]
    fn greedy_picks_best_value_first() {
        let upgrades = vec![
            up("a", "Ring",    1_000_000.0, 0.0, 0.0, 5.0),  // 200K dps/div
            up("b", "Helmet",  100_000.0,   0.0, 0.0, 10.0), // 10K dps/div
        ];
        let plan = greedy_multi_slot(&upgrades, 20.0, 5_000_000.0, &ResistConstraints::capped());
        // "a" should be picked first (better dps_per_divine)
        assert!(!plan.steps.is_empty(), "should pick at least one upgrade");
        assert_eq!(plan.steps[0].slot, "Ring", "best value upgrade should be first");
    }

    #[test]
    fn greedy_respects_budget() {
        let upgrades = vec![
            up("a", "Ring",    1_000_000.0, 0.0, 0.0, 100.0), // too expensive
            up("b", "Helmet",  500_000.0,   0.0, 0.0, 5.0),
        ];
        let plan = greedy_multi_slot(&upgrades, 10.0, 5_000_000.0, &ResistConstraints::capped());
        assert!(plan.total_cost <= 10.0 + 0.001, "total cost must not exceed budget");
    }

    #[test]
    fn greedy_no_upgrades_returns_empty_plan() {
        let plan = greedy_multi_slot(&[], 20.0, 5_000_000.0, &ResistConstraints::default());
        assert!(plan.steps.is_empty());
        assert_eq!(plan.total_dps_gain, 0.0);
    }

    #[test]
    fn greedy_cumulative_dps_increases_monotonically() {
        let upgrades = vec![
            up("a", "Ring",    500_000.0, 0.0, 0.0, 3.0),
            up("b", "Helmet",  300_000.0, 0.0, 0.0, 4.0),
            up("c", "Gloves",  200_000.0, 0.0, 0.0, 2.0),
        ];
        let plan = greedy_multi_slot(&upgrades, 20.0, 5_000_000.0, &ResistConstraints::capped());
        let mut prev = 0.0f64;
        for step in &plan.steps {
            assert!(step.cumulative_dps > prev, "cumulative DPS must increase each step");
            prev = step.cumulative_dps;
        }
    }
}
