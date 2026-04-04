/// stash.rs — Stash tab integration (Algorithm 38).
/// Tests written FIRST (TDD RED). Run `cargo test stash` → all FAIL → then implement.
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StashItem {
    pub id:              String,
    pub name:            String,
    pub type_line:       String,
    pub chaos_value:     f64,
    pub stack_size:      u32,
    pub tab_name:        String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StashUpgradeSuggestion {
    pub item_id:         String,
    pub item_name:       String,
    pub current_score:   f64,
    pub upgrade_score:   f64,
    pub score_gain:      f64,
    pub price_chaos:     f64,
    pub efficiency:      f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WealthSummary {
    pub total_chaos:     f64,
    pub total_divine:    f64,
    pub currency_map:    HashMap<String, f64>,
    pub total_items:     u32,
}

/// Token-bucket rate limiter (Algorithm 38: 45 requests / 60 seconds).
#[derive(Debug)]
pub struct RateLimiter {
    pub capacity:        u32,
    pub tokens:          f64,
    pub refill_rate:     f64,  // tokens per second
    pub last_refill_secs: f64,
}

// ─── Stubs → unimplemented!() → RED ──────────────────────────────────────────

impl RateLimiter {
    pub fn new(capacity: u32, requests_per_minute: u32) -> Self {
        RateLimiter {
            capacity,
            tokens: capacity as f64,
            refill_rate: requests_per_minute as f64 / 60.0,
            last_refill_secs: 0.0,
        }
    }

    pub fn try_acquire(&mut self, now_secs: f64) -> bool {
        // Refill tokens based on elapsed time
        let elapsed = (now_secs - self.last_refill_secs).max(0.0);
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill_secs = now_secs;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn wait_time_secs(&self, now_secs: f64) -> f64 {
        let elapsed = (now_secs - self.last_refill_secs).max(0.0);
        let current = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        if current >= 1.0 {
            0.0
        } else {
            (1.0 - current) / self.refill_rate
        }
    }
}

pub fn find_stash_upgrades(
    stash_items: &[StashItem],
    score_fn:    &dyn Fn(&StashItem) -> f64,
    upgrade_fn:  &dyn Fn(&StashItem) -> Option<f64>,
    min_gain:    f64,
) -> Vec<StashUpgradeSuggestion> {
    stash_items.iter().filter_map(|item| {
        let current = score_fn(item);
        let upgraded = upgrade_fn(item)?;
        let gain = upgraded - current;
        if gain < min_gain { return None; }
        let price = item.chaos_value;
        let efficiency = if price > 0.0 { gain / price } else { 0.0 };
        Some(StashUpgradeSuggestion {
            item_id:       item.id.clone(),
            item_name:     item.name.clone(),
            current_score: current,
            upgrade_score: upgraded,
            score_gain:    gain,
            price_chaos:   price,
            efficiency,
        })
    }).collect()
}

pub fn tally_currency(items: &[StashItem], divine_price_c: f64) -> WealthSummary {
    let mut currency_map: HashMap<String, f64> = HashMap::new();
    let mut total_chaos = 0.0f64;

    for item in items {
        let value = item.chaos_value * item.stack_size as f64;
        total_chaos += value;
        *currency_map.entry(item.name.clone()).or_insert(0.0) += item.stack_size as f64;
    }

    let total_divine = if divine_price_c > 0.0 { total_chaos / divine_price_c } else { 0.0 };

    WealthSummary {
        total_chaos,
        total_divine,
        currency_map,
        total_items: items.len() as u32,
    }
}

pub fn filter_sellable<'a>(items: &'a [StashItem], min_c: f64) -> Vec<&'a StashItem> {
    items.iter().filter(|it| it.chaos_value >= min_c).collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, name: &str, chaos: f64, stack: u32) -> StashItem {
        StashItem { id: id.to_string(), name: name.to_string(), type_line: name.to_string(),
            chaos_value: chaos, stack_size: stack, tab_name: "Tab1".to_string() }
    }

    // ── RateLimiter ───────────────────────────────────────────────────────────

    #[test]
    fn rate_limiter_starts_full() {
        let rl = RateLimiter::new(45, 45);
        assert!((rl.tokens - 45.0).abs() < 0.001, "should start at full capacity");
    }

    #[test]
    fn rate_limiter_acquires_token_when_available() {
        let mut rl = RateLimiter::new(45, 45);
        assert!(rl.try_acquire(0.0), "should succeed when tokens > 0");
    }

    #[test]
    fn rate_limiter_depletes_tokens() {
        let mut rl = RateLimiter::new(3, 3);
        rl.try_acquire(0.0);
        rl.try_acquire(0.0);
        rl.try_acquire(0.0);
        assert!(!rl.try_acquire(0.0), "4th acquire should fail (empty)");
    }

    #[test]
    fn rate_limiter_refills_over_time() {
        let mut rl = RateLimiter::new(3, 3);
        rl.try_acquire(0.0);
        rl.try_acquire(0.0);
        rl.try_acquire(0.0);
        // After 20s, ~1 token refilled (3 tokens/min = 1 token/20s)
        assert!(rl.try_acquire(21.0), "token should have refilled after 21s");
    }

    #[test]
    fn rate_limiter_respects_max_capacity() {
        let mut rl = RateLimiter::new(3, 3);
        // Even after 1000s, tokens should not exceed capacity
        rl.try_acquire(1000.0);
        assert!(rl.tokens <= 3.0, "tokens must not exceed capacity");
    }

    #[test]
    fn rate_limiter_wait_time_is_zero_when_tokens_available() {
        let rl = RateLimiter::new(45, 45);
        assert_eq!(rl.wait_time_secs(0.0), 0.0);
    }

    #[test]
    fn rate_limiter_wait_time_positive_when_empty() {
        let mut rl = RateLimiter::new(1, 1);
        rl.try_acquire(0.0);
        assert!(rl.wait_time_secs(0.0) > 0.0, "should have to wait when empty");
    }

    // ── find_stash_upgrades ───────────────────────────────────────────────────

    #[test]
    fn find_upgrades_returns_items_with_gain_above_threshold() {
        let items = vec![
            item("a", "Helmet", 50.0, 1),
            item("b", "Gloves", 20.0, 1),
        ];
        // helmet gains 10, gloves gains 3 — only helmet passes min_gain=5
        let suggestions = find_stash_upgrades(
            &items,
            &|it| if it.name == "Helmet" { 30.0 } else { 20.0 },
            &|it| if it.name == "Helmet" { Some(40.0) } else { Some(23.0) },
            5.0,
        );
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].item_name, "Helmet");
    }

    #[test]
    fn find_upgrades_computes_score_gain_correctly() {
        let items = vec![item("x", "Boots", 30.0, 1)];
        let suggestions = find_stash_upgrades(
            &items,
            &|_| 25.0,
            &|_| Some(35.0),
            5.0,
        );
        assert!(!suggestions.is_empty());
        assert!((suggestions[0].score_gain - 10.0).abs() < 0.001);
    }

    #[test]
    fn find_upgrades_skips_items_with_no_upgrade() {
        let items = vec![item("y", "Ring", 40.0, 1)];
        let suggestions = find_stash_upgrades(
            &items,
            &|_| 20.0,
            &|_| None,
            5.0,
        );
        assert!(suggestions.is_empty());
    }

    #[test]
    fn find_upgrades_efficiency_is_score_gain_over_price() {
        let items = vec![item("z", "Amulet", 100.0, 1)];
        let suggestions = find_stash_upgrades(
            &items,
            &|_| 10.0,
            &|_| Some(20.0),
            5.0,
        );
        assert!(!suggestions.is_empty());
        let s = &suggestions[0];
        let expected = s.score_gain / s.price_chaos;
        assert!((s.efficiency - expected).abs() < 0.001);
    }

    // ── tally_currency ────────────────────────────────────────────────────────

    #[test]
    fn tally_currency_sums_stack_values() {
        let items = vec![
            item("1", "Chaos Orb",  1.0, 100),
            item("2", "Exalted Orb", 50.0, 2),
        ];
        let summary = tally_currency(&items, 200.0);
        assert!((summary.total_chaos - 200.0).abs() < 0.001,
            "100×1 + 2×50 = 200c; got {}", summary.total_chaos);
    }

    #[test]
    fn tally_currency_converts_to_divine_correctly() {
        let items = vec![item("1", "Chaos Orb", 1.0, 400)];
        let summary = tally_currency(&items, 200.0);
        assert!((summary.total_divine - 2.0).abs() < 0.001,
            "400c / 200c per divine = 2 divines; got {}", summary.total_divine);
    }

    #[test]
    fn tally_currency_empty_items_returns_zero() {
        let summary = tally_currency(&[], 200.0);
        assert_eq!(summary.total_chaos, 0.0);
        assert_eq!(summary.total_items, 0);
    }

    #[test]
    fn tally_currency_item_count_matches() {
        let items = vec![item("a", "x", 1.0, 1), item("b", "y", 1.0, 1)];
        let summary = tally_currency(&items, 200.0);
        assert_eq!(summary.total_items, 2);
    }

    // ── filter_sellable ───────────────────────────────────────────────────────

    #[test]
    fn filter_sellable_excludes_below_threshold() {
        let items = vec![item("a", "Cheap", 1.0, 1), item("b", "Valuable", 50.0, 1)];
        let result = filter_sellable(&items, 10.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Valuable");
    }

    #[test]
    fn filter_sellable_includes_exact_threshold() {
        let items = vec![item("a", "Exactly10", 10.0, 1)];
        let result = filter_sellable(&items, 10.0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_sellable_empty_input_empty_output() {
        let result = filter_sellable(&[], 10.0);
        assert!(result.is_empty());
    }
}
