/// poe.ninja price cache with 5-min TTL and circuit breaker.
/// See ALGORITHMS.md Algorithm 21.
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use crate::models::market::{PriceResult, PriceConfidence};

const TTL: Duration = Duration::from_secs(300); // 5 minutes
const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;

struct CacheEntry {
    price: f64,
    listings: u32,
    fetched_at: Instant,
}

struct PriceCache {
    entries: HashMap<String, CacheEntry>,
    failures: u32,
    circuit_open: bool,
    circuit_open_until: Option<Instant>,
}

static CACHE: LazyLock<Mutex<PriceCache>> = LazyLock::new(|| {
    Mutex::new(PriceCache {
        entries: HashMap::new(),
        failures: 0,
        circuit_open: false,
        circuit_open_until: None,
    })
});

pub async fn get_prices(item_names: &[String]) -> Result<Vec<PriceResult>> {
    let mut results = Vec::new();

    for name in item_names {
        let result = get_single_price(name).await;
        results.push(result);
    }

    Ok(results)
}

async fn get_single_price(item_name: &str) -> PriceResult {
    // Check cache first
    {
        let cache = CACHE.lock().unwrap();
        if let Some(entry) = cache.entries.get(item_name) {
            let age = entry.fetched_at.elapsed();
            if age < TTL {
                let div_ratio = 200.0_f64;
                return PriceResult {
                    item_name: item_name.to_string(),
                    price_div: entry.price / div_ratio,
                    price_chaos: entry.price,
                    confidence: if entry.listings > 50 { PriceConfidence::High }
                                else if entry.listings > 10 { PriceConfidence::Medium }
                                else { PriceConfidence::Low },
                    listings: entry.listings,
                    cached: true,
                    cache_age_secs: age.as_secs(),
                };
            }
        }

        // Circuit breaker: if open, return guess
        if cache.circuit_open {
            if cache.circuit_open_until.map(|t| Instant::now() < t).unwrap_or(false) {
                return not_found(item_name);
            }
        }
    }

    // Fetch from poe.ninja
    match fetch_from_poe_ninja(item_name).await {
        Ok((price, listings)) => {
            let div_ratio = 200.0_f64; // fallback; real ratio from poe.ninja divine orb price
            let mut cache = CACHE.lock().unwrap();
            cache.failures = 0;
            cache.circuit_open = false;
            cache.entries.insert(item_name.to_string(), CacheEntry {
                price,
                listings,
                fetched_at: Instant::now(),
            });
            PriceResult {
                item_name: item_name.to_string(),
                price_div: price / div_ratio,
                price_chaos: price,
                confidence: if listings > 50 { PriceConfidence::High }
                            else { PriceConfidence::Medium },
                listings,
                cached: false,
                cache_age_secs: 0,
            }
        }
        Err(_) => {
            let mut cache = CACHE.lock().unwrap();
            cache.failures += 1;
            if cache.failures >= CIRCUIT_BREAKER_THRESHOLD {
                cache.circuit_open = true;
                cache.circuit_open_until = Some(Instant::now() + Duration::from_secs(60));
            }
            not_found(item_name)
        }
    }
}

async fn fetch_from_poe_ninja(item_name: &str) -> Result<(f64, u32)> {
    let league = get_current_league();
    let (chaos, _div, listings) = super::poe_ninja_client::get_item_price(item_name, &league).await?;
    Ok((chaos, listings))
}

fn get_current_league() -> String {
    std::env::var("POE_LEAGUE").unwrap_or_else(|_| "Settlers".to_string())
}

fn not_found(item_name: &str) -> PriceResult {
    PriceResult {
        item_name: item_name.to_string(),
        price_div: 0.0,
        price_chaos: 0.0,
        confidence: PriceConfidence::Guess,
        listings: 0,
        cached: false,
        cache_age_secs: 0,
    }
}

/// Compute confidence tier based on listing count (testable helper).
#[allow(dead_code)]
pub(crate) fn confidence_for_listings(listings: u32) -> PriceConfidence {
    if listings > 50 { PriceConfidence::High }
    else if listings > 10 { PriceConfidence::Medium }
    else { PriceConfidence::Low }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_result_has_guess_confidence() {
        let r = not_found("Kaom's Heart");
        assert_eq!(r.item_name, "Kaom's Heart");
        assert_eq!(r.price_chaos, 0.0);
        assert_eq!(r.price_div, 0.0);
        assert_eq!(r.listings, 0);
        assert!(matches!(r.confidence, PriceConfidence::Guess));
    }

    #[test]
    fn confidence_high_for_more_than_50_listings() {
        assert!(matches!(confidence_for_listings(51), PriceConfidence::High));
        assert!(matches!(confidence_for_listings(100), PriceConfidence::High));
    }

    #[test]
    fn confidence_medium_for_11_to_50_listings() {
        assert!(matches!(confidence_for_listings(11), PriceConfidence::Medium));
        assert!(matches!(confidence_for_listings(50), PriceConfidence::Medium));
    }

    #[test]
    fn confidence_low_for_10_or_fewer_listings() {
        assert!(matches!(confidence_for_listings(0), PriceConfidence::Low));
        assert!(matches!(confidence_for_listings(10), PriceConfidence::Low));
    }

    #[tokio::test]
    async fn get_prices_empty_input_returns_empty_vec() {
        let result = get_prices(&[]).await.unwrap();
        assert!(result.is_empty(), "Empty input should return empty vec");
    }

    #[tokio::test]
    async fn get_prices_returns_one_result_per_item() {
        // With circuit breaker logic, unknown items get a not-found result rather than error
        let names = vec!["ItemThatDoesNotExist_XYZ123".to_string()];
        let result = get_prices(&names).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].item_name, "ItemThatDoesNotExist_XYZ123");
    }
}
