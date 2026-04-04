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
    currency: &CurrencyInventory,
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
    let has_open_prefix = item.mods.iter().filter(|m| matches!(m.mod_type, crate::models::build::ModType::Prefix) && !m.text.is_empty()).count() < 3;
    let has_open_suffix = item.mods.iter().filter(|m| matches!(m.mod_type, crate::models::build::ModType::Suffix) && !m.text.is_empty()).count() < 3;

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

fn bench_craft_life() -> CraftSuggestion {
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

fn essence_life() -> CraftSuggestion {
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

fn chaos_spam_example() -> CraftSuggestion {
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

fn estimate_craft_cost(slot: &str) -> f64 {
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

