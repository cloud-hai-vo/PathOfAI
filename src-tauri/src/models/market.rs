use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResult {
    pub item_name: String,
    pub price_div: f64,
    pub price_chaos: f64,
    pub confidence: PriceConfidence,
    pub listings: u32,
    pub cached: bool,
    pub cache_age_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceConfidence {
    High,   // > 50 listings
    Medium, // 10-50 listings
    Low,    // < 10 listings
    Guess,  // not found, estimated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeResult {
    pub item_name: String,
    pub slot: String,
    pub price_div: f64,
    pub dps_gain: f64,
    pub life_gain: i32,
    pub efficiency: f64,
    pub trade_url: String,
    pub mod_highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraftSuggestion {
    pub method: CraftMethod,
    pub target_mod: String,
    pub probability: f64,           // 0.0-1.0 per attempt
    pub attempts_99pct: u32,        // attempts for 99% success
    pub expected_cost_chaos: f64,
    pub dps_gain: f64,
    pub verdict: CraftVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CraftMethod {
    BenchCraft,     // deterministic, p=1.0
    Essence,
    Chaos,
    Fossil,
    Harvest,
    Recombinator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CraftVerdict {
    BestOption,
    SafeOption,
    HighRisk,
    NotWorthIt,
}
