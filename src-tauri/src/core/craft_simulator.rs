/// craft_simulator.rs — Monte Carlo Craft Simulator (Algorithm 16).
///
/// Simulates many craft attempts to estimate cost distribution, success rate,
/// and optimal stopping point vs buying from trade.
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A simulated craft attempt result.
#[derive(Debug, Clone)]
enum SimOutcome {
    Success { cost: f64, attempts: u32 },
    ShouldHaveBought { cost: f64, attempts: u32, best_score: f64 },
    BudgetExhausted  { cost: f64, attempts: u32, best_score: f64 },
}

impl SimOutcome {
    fn is_success(&self) -> bool { matches!(self, SimOutcome::Success { .. }) }
    fn cost(&self) -> f64 {
        match self { SimOutcome::Success { cost, .. }
                   | SimOutcome::ShouldHaveBought { cost, .. }
                   | SimOutcome::BudgetExhausted  { cost, .. } => *cost }
    }
}

/// Input for a Monte Carlo simulation.
#[derive(Debug, Clone)]
pub struct SimInput {
    /// Per-attempt success probability (from Algorithm 15).
    pub probability:        f64,
    /// Minimum item score to accept (0-100).
    pub target_score:       f64,
    /// Item score distribution: each attempt, score is drawn from [min_score, max_score].
    pub min_score:          f64,
    pub max_score:          f64,
    /// Market price for equivalent item (in divine equivalents).
    pub buy_price:          f64,
    /// Maximum budget (in divine equivalents).
    pub max_budget:         f64,
    /// Cost per crafting attempt (in divine equivalents).
    pub cost_per_attempt:   f64,
    /// Number of simulations to run.
    pub simulations:        u32,
}

/// Recommendation from the simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CraftRecommendation {
    Craft,
    Buy,
    CraftThenBuy,
}

/// Output of the Monte Carlo simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimResult {
    pub expected_cost:        f64,
    pub median_cost:          f64,
    pub success_rate:         f64,   // 0.0–1.0
    pub percentile_90_cost:   f64,
    pub recommendation:       CraftRecommendation,
    pub optimal_stop_attempt: u32,
}

// ─── Algorithm ────────────────────────────────────────────────────────────────

/// Run the Monte Carlo simulation and return aggregated statistics.
pub fn run_simulation(input: &SimInput) -> SimResult {
    let n   = input.simulations.max(1);
    let mut outcomes: Vec<SimOutcome> = Vec::with_capacity(n as usize);
    let mut rng = LcgRng::new(0x9E3779B9); // deterministic seed for reproducibility

    for _ in 0..n {
        let outcome = simulate_one(input, &mut rng);
        outcomes.push(outcome);
    }

    let successes: Vec<f64> = outcomes.iter()
        .filter(|o| o.is_success())
        .map(|o| o.cost())
        .collect();

    let success_rate = successes.len() as f64 / n as f64;

    let (expected_cost, median_cost, percentile_90) = if successes.is_empty() {
        (input.max_budget, input.max_budget, input.max_budget)
    } else {
        let mut sorted = successes.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let median = sorted[sorted.len() / 2];
        let p90    = sorted[(sorted.len() * 9 / 10).min(sorted.len() - 1)];
        (mean, median, p90)
    };

    let recommendation = if input.buy_price > 0.0 {
        if expected_cost < input.buy_price * 0.7 {
            CraftRecommendation::Craft
        } else if expected_cost > input.buy_price * 1.3 {
            CraftRecommendation::Buy
        } else {
            CraftRecommendation::CraftThenBuy
        }
    } else {
        CraftRecommendation::Craft
    };

    let optimal_stop = find_optimal_stop(input, &outcomes);

    SimResult {
        expected_cost,
        median_cost,
        success_rate,
        percentile_90_cost: percentile_90,
        recommendation,
        optimal_stop_attempt: optimal_stop,
    }
}

/// Simulate one complete craft session until success, price exceeds buy price, or budget runs out.
fn simulate_one(input: &SimInput, rng: &mut LcgRng) -> SimOutcome {
    let mut total_cost = 0.0f64;
    let mut best_score = 0.0f64;
    let mut attempts   = 0u32;

    loop {
        // Roll a random item score in [min_score, max_score].
        let score = input.min_score + rng.next_f64() * (input.max_score - input.min_score);
        if score > best_score { best_score = score; }
        attempts   += 1;
        total_cost += input.cost_per_attempt;

        if best_score >= input.target_score {
            return SimOutcome::Success { cost: total_cost, attempts };
        }
        if input.buy_price > 0.0 && total_cost >= input.buy_price {
            return SimOutcome::ShouldHaveBought { cost: total_cost, attempts, best_score };
        }
        if total_cost >= input.max_budget {
            return SimOutcome::BudgetExhausted { cost: total_cost, attempts, best_score };
        }
        // Safety cap: 10,000 attempts.
        if attempts >= 10_000 { break; }
    }

    SimOutcome::BudgetExhausted { cost: total_cost, attempts, best_score }
}

/// Find the attempt number where marginal improvement < marginal cost.
fn find_optimal_stop(input: &SimInput, outcomes: &[SimOutcome]) -> u32 {
    let n = outcomes.len() as f64;
    if n == 0.0 { return 1; }

    for attempt in 1u32..=100 {
        let successes_by = outcomes.iter()
            .filter(|o| matches!(o, SimOutcome::Success { attempts: a, .. } if *a <= attempt))
            .count() as f64 / n;
        let prev_successes = if attempt > 1 {
            outcomes.iter()
                .filter(|o| matches!(o, SimOutcome::Success { attempts: a, .. } if *a <= attempt - 1))
                .count() as f64 / n
        } else { 0.0 };

        let marginal_success = successes_by - prev_successes;
        let marginal_value   = marginal_success * input.buy_price;

        if marginal_value < input.cost_per_attempt {
            return attempt;
        }
    }

    50 // conservative fallback
}

// ─── Minimal LCG PRNG (no std rand dependency) ───────────────────────────────

struct LcgRng(u64);

impl LcgRng {
    fn new(seed: u64) -> Self { Self(seed) }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn easy_input() -> SimInput {
        SimInput {
            probability: 0.50,
            target_score: 60.0,
            min_score: 40.0, max_score: 90.0,
            buy_price: 10.0,
            max_budget: 50.0,
            cost_per_attempt: 1.0,
            simulations: 1000,
        }
    }

    fn hard_input() -> SimInput {
        SimInput {
            probability: 0.01,
            target_score: 90.0,
            min_score: 0.0, max_score: 100.0,
            buy_price: 5.0,
            max_budget: 5.0,
            cost_per_attempt: 1.0,
            simulations: 1000,
        }
    }

    #[test]
    fn easy_craft_has_high_success_rate() {
        let result = run_simulation(&easy_input());
        assert!(result.success_rate > 0.5, "easy craft should succeed > 50%: {}", result.success_rate);
    }

    #[test]
    fn hard_craft_has_low_success_rate() {
        let result = run_simulation(&hard_input());
        assert!(result.success_rate < 0.5, "hard craft with tight budget should fail often: {}", result.success_rate);
    }

    #[test]
    fn success_rate_is_between_zero_and_one() {
        let result = run_simulation(&easy_input());
        assert!(result.success_rate >= 0.0 && result.success_rate <= 1.0);
    }

    #[test]
    fn median_cost_does_not_exceed_expected_cost_much() {
        let result = run_simulation(&easy_input());
        // For a symmetric distribution median ≤ mean; for geometric/skewed median < mean.
        // We just check they're in the same ballpark (within 3×).
        assert!(result.median_cost <= result.expected_cost * 3.0 + 1.0,
            "median {} too far from mean {}", result.median_cost, result.expected_cost);
    }

    #[test]
    fn percentile_90_is_at_least_median() {
        let result = run_simulation(&easy_input());
        assert!(result.percentile_90_cost >= result.median_cost - 0.001,
            "90th percentile {} < median {}", result.percentile_90_cost, result.median_cost);
    }

    #[test]
    fn cheap_craft_recommends_craft() {
        // Make craft very cheap (1c per attempt, buy = 100d)
        let input = SimInput {
            probability: 0.9,
            target_score: 50.0,
            min_score: 40.0, max_score: 90.0,
            buy_price: 100.0,
            max_budget: 200.0,
            cost_per_attempt: 0.01,
            simulations: 500,
        };
        let result = run_simulation(&input);
        assert_eq!(result.recommendation, CraftRecommendation::Craft);
    }

    #[test]
    fn expensive_craft_recommends_buy() {
        // Craft costs 50d per attempt, buy = 1d
        let input = SimInput {
            probability: 0.01,
            target_score: 99.0,
            min_score: 0.0, max_score: 100.0,
            buy_price: 1.0,
            max_budget: 200.0,
            cost_per_attempt: 50.0,
            simulations: 500,
        };
        let result = run_simulation(&input);
        assert_eq!(result.recommendation, CraftRecommendation::Buy);
    }

    #[test]
    fn optimal_stop_is_at_least_one() {
        let result = run_simulation(&easy_input());
        assert!(result.optimal_stop_attempt >= 1);
    }

    #[test]
    fn single_simulation_still_works() {
        let mut i = easy_input();
        i.simulations = 1;
        let result = run_simulation(&i);
        assert!(result.success_rate >= 0.0 && result.success_rate <= 1.0);
    }

    #[test]
    fn lcg_rng_produces_values_in_range() {
        let mut rng = LcgRng::new(42);
        for _ in 0..100 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0, "LCG f64 must be in [0, 1): {v}");
        }
    }
}
