/// Price Poller — polls poe.ninja on a configurable interval and caches results.
/// Algorithm 21 (poe.ninja price cache): 5-minute TTL, stale fallback, circuit breaker.
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Tracks freshness and circuit-breaker state for the price cache.
pub struct PricePoller {
    cache:           HashMap<String, CacheEntry>,
    ttl:             Duration,
    circuit_open:    bool,
    failure_count:   u32,
    failure_threshold: u32,
    last_failure:    Option<Instant>,
    circuit_reset_s: u64,
}

#[derive(Clone)]
pub struct CacheEntry {
    pub price_div:  f64,
    pub fetched_at: Instant,
}

impl PricePoller {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(ttl_secs),
            circuit_open: false,
            failure_count: 0,
            failure_threshold: 3,
            last_failure: None,
            circuit_reset_s: 60,
        }
    }

    /// Insert or update a price entry in the cache.
    pub fn insert(&mut self, item: &str, price_div: f64) {
        self.cache.insert(item.to_string(), CacheEntry {
            price_div,
            fetched_at: Instant::now(),
        });
        // Reset circuit on successful fetch
        self.failure_count = 0;
        self.circuit_open  = false;
    }

    /// Get a cached price if fresh (within TTL). Returns stale price if circuit open.
    pub fn get(&self, item: &str) -> Option<f64> {
        self.cache.get(item).and_then(|entry| {
            if entry.fetched_at.elapsed() < self.ttl || self.circuit_open {
                Some(entry.price_div)
            } else {
                None
            }
        })
    }

    /// Returns true if the cached value is stale (past TTL) and should be refreshed.
    pub fn is_stale(&self, item: &str) -> bool {
        match self.cache.get(item) {
            None => true,
            Some(entry) => entry.fetched_at.elapsed() >= self.ttl,
        }
    }

    /// Record an API fetch failure. Opens the circuit after threshold failures.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure   = Some(Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.circuit_open = true;
        }
    }

    /// Returns true if the circuit is open (API requests should be skipped).
    pub fn circuit_is_open(&self) -> bool {
        if !self.circuit_open { return false; }
        // Auto-reset after circuit_reset_s
        if let Some(t) = self.last_failure {
            if t.elapsed().as_secs() >= self.circuit_reset_s {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_price_returned_within_ttl() {
        let mut p = PricePoller::new(300);
        p.insert("Watcher's Eye", 8.5);
        assert_eq!(p.get("Watcher's Eye"), Some(8.5));
    }

    #[test]
    fn missing_item_returns_none() {
        let p = PricePoller::new(300);
        assert_eq!(p.get("Unknown Item"), None);
    }

    #[test]
    fn stale_entry_returns_none_when_circuit_closed() {
        let mut p = PricePoller::new(0); // 0s TTL — always stale
        p.insert("Watcher's Eye", 8.5);
        // Immediately stale — should return None when circuit is closed
        assert_eq!(p.get("Watcher's Eye"), None);
    }

    #[test]
    fn stale_entry_returns_fallback_when_circuit_open() {
        let mut p = PricePoller::new(0); // 0s TTL — immediately stale
        p.insert("Watcher's Eye", 8.5);
        // Open the circuit
        p.record_failure();
        p.record_failure();
        p.record_failure();
        assert!(p.circuit_is_open());
        // Stale fallback: even though TTL expired, circuit open → return cached value
        assert_eq!(p.get("Watcher's Eye"), Some(8.5));
    }

    #[test]
    fn is_stale_true_for_missing_item() {
        let p = PricePoller::new(300);
        assert!(p.is_stale("New Item"));
    }

    #[test]
    fn is_stale_false_within_ttl() {
        let mut p = PricePoller::new(300);
        p.insert("Item", 1.0);
        assert!(!p.is_stale("Item"));
    }

    #[test]
    fn circuit_opens_after_threshold_failures() {
        let mut p = PricePoller::new(300);
        assert!(!p.circuit_is_open());
        p.record_failure();
        p.record_failure();
        assert!(!p.circuit_is_open()); // 2 < threshold (3)
        p.record_failure();
        assert!(p.circuit_is_open()); // 3 >= threshold
    }

    #[test]
    fn successful_insert_resets_circuit() {
        let mut p = PricePoller::new(300);
        p.record_failure();
        p.record_failure();
        p.record_failure();
        assert!(p.circuit_is_open());
        p.insert("Item", 1.0);
        assert!(!p.circuit_is_open());
    }

    #[test]
    fn insert_overwrites_old_entry() {
        let mut p = PricePoller::new(300);
        p.insert("Item", 5.0);
        p.insert("Item", 7.5);
        assert_eq!(p.get("Item"), Some(7.5));
    }
}
