/// poe.ninja API client — Algorithm 21 (Price Cache).
/// Fetches bulk price lists per category; individual lookups hit the cache.
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const BASE_URL: &str = "https://poe.ninja/api/data";
const CATEGORY_TTL: Duration = Duration::from_secs(300); // 5 min

// --- types returned by the API ---

#[derive(Debug, Deserialize)]
struct NinjaItemLine {
    name: String,
    #[serde(rename = "chaosValue", default)]
    chaos_value: f64,
    #[serde(rename = "divineValue", default)]
    divine_value: f64,
    #[serde(rename = "listingCount", default)]
    listing_count: u32,
}

#[derive(Debug, Deserialize)]
struct NinjaCurrencyLine {
    #[serde(rename = "currencyTypeName")]
    currency_type_name: String,
    #[serde(rename = "chaosEquivalent", default)]
    chaos_equivalent: f64,
}

#[derive(Debug, Deserialize)]
struct NinjaItemResponse {
    lines: Vec<NinjaItemLine>,
}

#[derive(Debug, Deserialize)]
struct NinjaCurrencyResponse {
    lines: Vec<NinjaCurrencyLine>,
}

// --- category list cache ---

struct CategoryCache {
    /// name → (chaos, div, listings)
    prices: HashMap<String, (f64, f64, u32)>,
    fetched_at: Instant,
}

static CATEGORY_CACHES: LazyLock<Mutex<HashMap<String, CategoryCache>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Fetch a single item price by name.
/// Tries each PoE item category in order until found.
/// Returns (chaos_value, divine_value, listing_count).
pub async fn get_item_price(
    item_name: &str,
    league: &str,
) -> Result<(f64, f64, u32)> {
    let categories = [
        "UniqueArmour",
        "UniqueWeapon",
        "UniqueAccessory",
        "UniqueJewel",
        "UniqueFlask",
        "UniqueMap",
        "DivinationCard",
        "SkillGem",
    ];

    for category in &categories {
        if let Ok(Some(price)) = lookup_in_cache(item_name, category, league).await {
            return Ok(price);
        }
    }

    // Try currency
    if let Ok(Some(price)) = lookup_currency(item_name, league).await {
        return Ok(price);
    }

    Err(anyhow!("Item '{}' not found on poe.ninja", item_name))
}

/// Get the chaos/divine ratio (price of 1 divine orb in chaos).
pub async fn get_divine_ratio(league: &str) -> Result<f64> {
    let cache_key = format!("currency|{league}");
    ensure_currency_cache(league, &cache_key).await?;

    let cache = CATEGORY_CACHES.lock().unwrap();
    if let Some(cat) = cache.get(&cache_key) {
        if let Some(&(chaos, _, _)) = cat.prices.get("Divine Orb") {
            if chaos > 0.0 { return Ok(chaos); }
        }
    }
    Ok(200.0) // fallback
}

// --- internal helpers ---

async fn lookup_in_cache(
    item_name: &str,
    category: &str,
    league: &str,
) -> Result<Option<(f64, f64, u32)>> {
    let cache_key = format!("{category}|{league}");
    ensure_item_cache(category, league, &cache_key).await?;

    let cache = CATEGORY_CACHES.lock().unwrap();
    Ok(cache.get(&cache_key).and_then(|c| c.prices.get(item_name).copied()))
}

async fn lookup_currency(
    item_name: &str,
    league: &str,
) -> Result<Option<(f64, f64, u32)>> {
    let cache_key = format!("currency|{league}");
    ensure_currency_cache(league, &cache_key).await?;

    let cache = CATEGORY_CACHES.lock().unwrap();
    Ok(cache.get(&cache_key).and_then(|c| c.prices.get(item_name).copied()))
}

async fn ensure_item_cache(category: &str, league: &str, cache_key: &str) -> Result<()> {
    let needs_refresh = {
        let cache = CATEGORY_CACHES.lock().unwrap();
        match cache.get(cache_key) {
            Some(c) => c.fetched_at.elapsed() > CATEGORY_TTL,
            None => true,
        }
    };

    if needs_refresh {
        let url = format!(
            "{BASE_URL}/itemoverview?league={league}&type={category}",
            league = urlencoding::encode(league)
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let resp = client.get(&url).send().await?;
        let data: NinjaItemResponse = resp.json().await?;

        let mut prices = HashMap::new();
        for line in data.lines {
            prices.insert(line.name, (line.chaos_value, line.divine_value, line.listing_count));
        }

        let mut cache = CATEGORY_CACHES.lock().unwrap();
        cache.insert(cache_key.to_string(), CategoryCache {
            prices,
            fetched_at: Instant::now(),
        });
    }

    Ok(())
}

async fn ensure_currency_cache(league: &str, cache_key: &str) -> Result<()> {
    let needs_refresh = {
        let cache = CATEGORY_CACHES.lock().unwrap();
        match cache.get(cache_key) {
            Some(c) => c.fetched_at.elapsed() > CATEGORY_TTL,
            None => true,
        }
    };

    if needs_refresh {
        let url = format!(
            "{BASE_URL}/currencyoverview?league={league}&type=Currency",
            league = urlencoding::encode(league)
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let resp = client.get(&url).send().await?;
        let data: NinjaCurrencyResponse = resp.json().await?;

        // chaos equivalent = how many chaos 1 unit of currency is worth
        let divine_ratio = data.lines.iter()
            .find(|l| l.currency_type_name == "Divine Orb")
            .map(|l| l.chaos_equivalent)
            .unwrap_or(200.0);

        let mut prices = HashMap::new();
        for line in data.lines {
            let chaos = line.chaos_equivalent;
            let div = if divine_ratio > 0.0 { chaos / divine_ratio } else { 0.0 };
            prices.insert(line.currency_type_name, (chaos, div, 0u32));
        }

        let mut cache = CATEGORY_CACHES.lock().unwrap();
        cache.insert(cache_key.to_string(), CategoryCache {
            prices,
            fetched_at: Instant::now(),
        });
    }

    Ok(())
}

// ─── Testable pure helpers ────────────────────────────────────────────────────

/// Build item URL for a given category and league (testable helper).
pub(crate) fn build_item_url(category: &str, league: &str) -> String {
    format!("{BASE_URL}/itemoverview?league={}&type={category}", urlencoding::encode(league))
}

/// Build currency URL for a league (testable helper).
pub(crate) fn build_currency_url(league: &str) -> String {
    format!("{BASE_URL}/currencyoverview?league={}&type=Currency", urlencoding::encode(league))
}

/// Parse raw item data into a price map (name → (chaos, div, listings)).
pub(crate) fn build_price_map_from_items(
    items: &[(&str, f64, f64, u32)],
) -> HashMap<String, (f64, f64, u32)> {
    items.iter()
        .map(|&(name, chaos, div, listings)| (name.to_string(), (chaos, div, listings)))
        .collect()
}

/// Parse currency data into a price map given the divine orb chaos price.
pub(crate) fn build_price_map_from_currency(
    currencies: &[(&str, f64)],
    divine_chaos: f64,
) -> HashMap<String, (f64, f64, u32)> {
    currencies.iter()
        .map(|&(name, chaos)| {
            let div = if divine_chaos > 0.0 { chaos / divine_chaos } else { 0.0 };
            (name.to_string(), (chaos, div, 0u32))
        })
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_url_contains_league_and_category() {
        let url = build_item_url("UniqueArmour", "Settlers");
        assert!(url.contains("itemoverview"), "URL should be for itemoverview");
        assert!(url.contains("UniqueArmour"), "URL should contain category");
        assert!(url.contains("Settlers"), "URL should contain league");
    }

    #[test]
    fn currency_url_contains_league() {
        let url = build_currency_url("Settlers");
        assert!(url.contains("currencyoverview"), "URL should be for currencyoverview");
        assert!(url.contains("Settlers"), "URL should contain league");
        assert!(url.contains("Currency"), "URL should include Currency type");
    }

    #[test]
    fn item_url_encodes_spaces_in_league_name() {
        let url = build_item_url("UniqueWeapon", "Standard League");
        assert!(url.contains("Standard%20League") || url.contains("Standard+League"),
            "URL should encode spaces: {url}");
    }

    #[test]
    fn build_price_map_from_items_stores_all_fields() {
        let items = [("Kaom's Heart", 800.0, 4.0, 120u32)];
        let map = build_price_map_from_items(&items);
        let &(chaos, div, listings) = map.get("Kaom's Heart").unwrap();
        assert_eq!(chaos, 800.0);
        assert_eq!(div, 4.0);
        assert_eq!(listings, 120);
    }

    #[test]
    fn build_price_map_from_currency_computes_div_ratio() {
        // 1 divine = 200 chaos. An orb worth 10c should be 0.05 div.
        let currencies = [("Chaos Orb", 1.0), ("Divine Orb", 200.0), ("Vaal Orb", 10.0)];
        let map = build_price_map_from_currency(&currencies, 200.0);

        let &(chaos, div, _) = map.get("Vaal Orb").unwrap();
        assert_eq!(chaos, 10.0);
        assert!((div - 0.05).abs() < 0.001, "Expected 0.05 div, got {div}");
    }

    #[test]
    fn build_price_map_from_currency_zero_divine_ratio_returns_zero_div() {
        let currencies = [("Some Item", 50.0)];
        let map = build_price_map_from_currency(&currencies, 0.0);
        let &(_, div, _) = map.get("Some Item").unwrap();
        assert_eq!(div, 0.0, "Zero divine ratio should produce 0.0 div");
    }
}
