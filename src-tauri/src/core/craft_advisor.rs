/// Crafting advisor — see ALGORITHMS.md Algorithm 24 and Algorithm 47.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::models::build::{BuildData, Item, ItemRarity};
use crate::models::market::{CraftSuggestion, CraftMethod, CraftVerdict};
use crate::calculator::formulas::geometric_99th_percentile;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CurrencyInventory {
    pub chaos: f64,
    pub divine: f64,
    pub exalted: f64,
    pub essence_count: u32,
    pub fossil_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CraftVsBuyResult {
    pub slot: String,
    pub current_item: String,
    pub craft_cost_div: f64,
    pub buy_cost_div: f64,
    pub verdict: CraftVerdict,
    pub recommendation: String,
}

/// Get craft suggestions for the build based on archetype needs.
pub fn get_suggestions(
    build: &BuildData,
    _currency: &CurrencyInventory,
) -> Result<Vec<CraftSuggestion>> {
    let mut suggestions: Vec<CraftSuggestion> = Vec::new();

    // Analyze each item for crafting opportunities
    for item in &build.items {
        if item.slot.is_empty() { continue; }
        if matches!(item.rarity, ItemRarity::Unique) { continue; } // can't craft on uniques

        suggest_for_item(item, &mut suggestions);
    }

    // Sort by verdict (BestOption first), then by expected cost
    suggestions.sort_by(|a, b| {
        let verdict_rank = |v: &CraftVerdict| match v {
            CraftVerdict::BestOption => 0,
            CraftVerdict::SafeOption => 1,
            CraftVerdict::HighRisk => 2,
            CraftVerdict::NotWorthIt => 3,
        };
        verdict_rank(&a.verdict).cmp(&verdict_rank(&b.verdict))
            .then(a.expected_cost_chaos.partial_cmp(&b.expected_cost_chaos).unwrap_or(std::cmp::Ordering::Equal))
    });

    // If no item suggestions, add defaults
    if suggestions.is_empty() {
        suggestions.push(bench_craft_life());
        suggestions.push(essence_life());
        suggestions.push(chaos_spam_example());
    }

    Ok(suggestions)
}

fn suggest_for_item(item: &Item, suggestions: &mut Vec<CraftSuggestion>) {
    let slot = item.slot.as_str();
    let has_life = item.mods.iter().any(|m| {
        let t = m.text.to_lowercase();
        t.contains("maximum life") || t.contains("to maximum life")
    });
    let _has_open_prefix = item.mods.iter().filter(|m| matches!(m.mod_type, crate::models::build::ModType::Prefix) && !m.text.is_empty()).count() < 3;
    let _has_open_suffix = item.mods.iter().filter(|m| matches!(m.mod_type, crate::models::build::ModType::Suffix) && !m.text.is_empty()).count() < 3;

    // Bench craft life if missing and slot supports it
    if !has_life && bench_craft_slots().contains(&slot) {
        let mut s = bench_craft_life();
        s.target_mod = format!("{} — +life mod", item.slot);
        s.dps_gain = 0.0;
        suggestions.push(s);
    }

    // Suggest essence spam for items with few mods
    if item.mods.len() < 4 && should_suggest_essence(slot) {
        suggestions.push(essence_for_slot(slot));
    }

    // Suggest chaos spam if item has bad rolls
    if item.mods.len() >= 3 && !has_life && matches!(item.rarity, ItemRarity::Rare) {
        suggestions.push(chaos_spam_example());
    }
}

fn bench_craft_slots() -> &'static [&'static str] {
    &["Helmet", "BodyArmour", "Gloves", "Boots", "Belt", "Amulet", "Ring", "Ring 1", "Ring 2", "Weapon", "Shield"]
}

fn should_suggest_essence(slot: &str) -> bool {
    matches!(slot, "Helmet" | "BodyArmour" | "Gloves" | "Boots" | "Weapon" | "Shield")
}

pub(crate) fn bench_craft_life() -> CraftSuggestion {
    // Benchcraft life is deterministic — always succeeds on open prefix
    CraftSuggestion {
        method: CraftMethod::BenchCraft,
        target_mod: "Maximum Life (+60-79)".to_string(),
        probability: 1.0,
        attempts_99pct: 1,
        expected_cost_chaos: 2.0,
        dps_gain: 0.0,
        verdict: CraftVerdict::BestOption,
    }
}

pub(crate) fn essence_life() -> CraftSuggestion {
    // Essence of Greed — guarantees life mod, other mods random
    // ~1/10 chance to hit a useful second mod (conservative)
    let p = 0.10_f64;
    let attempts = geometric_99th_percentile(p);
    let chaos_per_attempt = 5.0; // ~5c per Essence of Greed
    CraftSuggestion {
        method: CraftMethod::Essence,
        target_mod: "Life + useful second mod (Essence of Greed)".to_string(),
        probability: p,
        attempts_99pct: attempts,
        expected_cost_chaos: (1.0 / p) * chaos_per_attempt,
        dps_gain: 0.0,
        verdict: CraftVerdict::SafeOption,
    }
}

fn essence_for_slot(slot: &str) -> CraftSuggestion {
    let (essence_name, p, cost_per) = match slot {
        "Weapon" => ("Essence of Hysteria (fire dot multi)", 0.05, 30.0),
        "Helmet" => ("Essence of Zeal (attack speed)", 0.08, 10.0),
        "Gloves" => ("Essence of Hatred (cold dmg)", 0.10, 8.0),
        _ => ("Essence of Greed (life)", 0.10, 5.0),
    };
    let attempts = geometric_99th_percentile(p);
    CraftSuggestion {
        method: CraftMethod::Essence,
        target_mod: format!("{essence_name} + good second mod"),
        probability: p,
        attempts_99pct: attempts,
        expected_cost_chaos: (1.0 / p) * cost_per,
        dps_gain: 0.0,
        verdict: if p >= 0.10 { CraftVerdict::SafeOption } else { CraftVerdict::HighRisk },
    }
}

pub(crate) fn chaos_spam_example() -> CraftSuggestion {
    // Chaos orb: ~1/200 to hit a 6-mod rare with good mods on a 4-affix item
    let p = 0.005_f64;
    let attempts = geometric_99th_percentile(p);
    CraftSuggestion {
        method: CraftMethod::Chaos,
        target_mod: "Reroll rare item for better mods".to_string(),
        probability: p,
        attempts_99pct: attempts,
        expected_cost_chaos: (1.0 / p) * 1.0,
        dps_gain: 0.0,
        verdict: CraftVerdict::HighRisk,
    }
}

/// Compare expected crafting cost vs buying the item outright.
pub fn compare_craft_vs_buy(
    slot: &str,
    build: &BuildData,
    buy_price_div: f64,
) -> CraftVsBuyResult {
    // Estimate craft cost for this slot
    let craft_cost_chaos = estimate_craft_cost(slot);
    let divine_ratio = 200.0_f64; // approximate
    let craft_cost_div = craft_cost_chaos / divine_ratio;

    let current_item_name = build.items.iter()
        .find(|i| i.slot.eq_ignore_ascii_case(slot))
        .map(|i| i.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let (verdict, recommendation) = if buy_price_div <= 0.0 {
        (CraftVerdict::BestOption, format!("Craft it — no price data for market comparison"))
    } else if craft_cost_div < buy_price_div * 0.7 {
        (CraftVerdict::BestOption, format!("Craft is ~{:.0}% cheaper — craft first", (1.0 - craft_cost_div / buy_price_div) * 100.0))
    } else if craft_cost_div > buy_price_div * 1.3 {
        (CraftVerdict::NotWorthIt, format!("Buy it — {:.1} div vs ~{:.1} div to craft", buy_price_div, craft_cost_div))
    } else {
        (CraftVerdict::SafeOption, format!("Similar cost — craft gives control over mods"))
    };

    CraftVsBuyResult {
        slot: slot.to_string(),
        current_item: current_item_name,
        craft_cost_div,
        buy_cost_div: buy_price_div,
        verdict,
        recommendation,
    }
}

pub(crate) fn estimate_craft_cost(slot: &str) -> f64 {
    // Expected chaos to reach a "good" rare via essence spam
    match slot {
        "Weapon" => 200.0,    // weapon crafts are expensive
        "BodyArmour" => 150.0,
        "Helmet" => 100.0,
        "Gloves" | "Boots" => 80.0,
        "Belt" => 60.0,
        "Amulet" | "Ring" | "Ring 1" | "Ring 2" => 50.0,
        _ => 100.0,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculator::formulas::geometric_99th_percentile;
    use crate::models::build::BuildData;
    use crate::models::market::{CraftMethod, CraftVerdict};

    // ── bench_craft_life ──────────────────────────────────────────────────────

    #[test]
    fn bench_craft_life_is_deterministic_p1() {
        let s = bench_craft_life();
        assert_eq!(s.probability, 1.0, "BenchCraft must have p=1.0");
        assert_eq!(s.attempts_99pct, 1, "p=1.0 deterministic means 1 attempt");
        assert_eq!(s.expected_cost_chaos, 2.0, "Bench craft costs ~2c");
        assert!(matches!(s.verdict, CraftVerdict::BestOption));
        assert!(matches!(s.method, CraftMethod::BenchCraft));
    }

    // ── essence_life ──────────────────────────────────────────────────────────

    #[test]
    fn essence_life_probability_and_cost_match() {
        let s = essence_life();
        assert!((s.probability - 0.10).abs() < 0.001, "Essence should have p≈0.10");
        let expected_cost = (1.0 / 0.10) * 5.0; // 1/p * cost_per_attempt
        assert!((s.expected_cost_chaos - expected_cost).abs() < 0.01,
            "Expected {expected_cost}c, got {}c", s.expected_cost_chaos);
        assert!(matches!(s.verdict, CraftVerdict::SafeOption));
    }

    #[test]
    fn essence_life_attempts_match_geometric_formula() {
        let s = essence_life();
        assert_eq!(s.attempts_99pct, geometric_99th_percentile(0.10),
            "attempts_99pct should match geometric_99th_percentile(0.10)");
    }

    // ── chaos_spam ────────────────────────────────────────────────────────────

    #[test]
    fn chaos_spam_is_high_risk_with_many_attempts() {
        let s = chaos_spam_example();
        assert!(matches!(s.verdict, CraftVerdict::HighRisk));
        assert!(s.probability < 0.01, "Chaos spam probability should be very low");
        assert!(s.attempts_99pct > 100, "Chaos spam needs many attempts");
        assert!(matches!(s.method, CraftMethod::Chaos));
    }

    // ── estimate_craft_cost ───────────────────────────────────────────────────

    #[test]
    fn weapon_craft_is_most_expensive() {
        let weapon_cost = estimate_craft_cost("Weapon");
        assert!(weapon_cost > estimate_craft_cost("Ring"), "Weapons cost more than rings");
        assert!(weapon_cost > estimate_craft_cost("Boots"), "Weapons cost more than boots");
        assert!(weapon_cost > estimate_craft_cost("Belt"), "Weapons cost more than belts");
    }

    #[test]
    fn unknown_slot_has_default_cost() {
        let cost = estimate_craft_cost("Unknown Slot");
        assert_eq!(cost, 100.0, "Unknown slots should have 100c default cost");
    }

    // ── compare_craft_vs_buy ──────────────────────────────────────────────────

    #[test]
    fn compare_craft_wins_when_much_cheaper_than_buy() {
        // Boots craft = 80c = 0.4 div; buy = 5 div → craft is >>30% cheaper
        let build = BuildData::default();
        let result = compare_craft_vs_buy("Boots", &build, 5.0);
        assert!(matches!(result.verdict, CraftVerdict::BestOption),
            "Crafting should win when craft costs 0.4div vs buy 5div");
        assert!(result.recommendation.contains("craft"), "Recommendation should mention crafting");
    }

    #[test]
    fn compare_buy_wins_when_much_cheaper_than_craft() {
        // Weapon craft = 200c = 1 div; buy = 0.05 div → buy is >>30% cheaper
        let build = BuildData::default();
        let result = compare_craft_vs_buy("Weapon", &build, 0.05);
        assert!(matches!(result.verdict, CraftVerdict::NotWorthIt),
            "Buying should win when buy=0.05div vs craft=1div");
        assert!(result.recommendation.contains("Buy"), "Recommendation should mention buying");
    }

    #[test]
    fn compare_similar_cost_is_safe_option() {
        // Belt craft = 60c = 0.3 div; buy = 0.3 div → within 30% margin
        let build = BuildData::default();
        let result = compare_craft_vs_buy("Belt", &build, 0.3);
        assert!(matches!(result.verdict, CraftVerdict::SafeOption),
            "Similar cost should give SafeOption verdict");
    }

    #[test]
    fn compare_no_price_data_defaults_to_craft() {
        let build = BuildData::default();
        let result = compare_craft_vs_buy("Helmet", &build, 0.0);
        assert!(matches!(result.verdict, CraftVerdict::BestOption),
            "No market price → default to crafting");
    }

    // ── get_suggestions ───────────────────────────────────────────────────────

    #[test]
    fn get_suggestions_empty_build_returns_default_suggestions() {
        let build = BuildData::default();
        let currency = CurrencyInventory::default();
        let result = get_suggestions(&build, &currency).unwrap();
        assert!(!result.is_empty(), "Should return default suggestions for empty build");
        assert!(result.iter().any(|s| matches!(s.method, CraftMethod::BenchCraft)),
            "Default suggestions should include BenchCraft");
    }

    #[test]
    fn get_suggestions_sorts_best_option_first() {
        let build = BuildData::default();
        let currency = CurrencyInventory::default();
        let result = get_suggestions(&build, &currency).unwrap();
        if result.len() > 1 {
            let first_rank = match &result[0].verdict {
                CraftVerdict::BestOption => 0,
                CraftVerdict::SafeOption => 1,
                CraftVerdict::HighRisk => 2,
                CraftVerdict::NotWorthIt => 3,
            };
            let second_rank = match &result[1].verdict {
                CraftVerdict::BestOption => 0,
                CraftVerdict::SafeOption => 1,
                CraftVerdict::HighRisk => 2,
                CraftVerdict::NotWorthIt => 3,
            };
            assert!(first_rank <= second_rank, "Suggestions should be sorted best-first");
        }
    }
}
