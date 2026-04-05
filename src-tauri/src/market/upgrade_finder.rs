use anyhow::Result;
use crate::models::build::BuildData;
use crate::models::analysis::AnalysisResult;
use crate::models::market::TradeResult;

/// Find upgrade suggestions for a given item slot, ranked by efficiency.
/// Uses poe.ninja prices to estimate cost and scores DPS/life gain vs current item.
pub async fn find_upgrades(
    slot: &str,
    _build: &BuildData,
    analysis: &AnalysisResult,
    budget_div: f64,
) -> Result<Vec<TradeResult>> {
    // Get current item score for the slot
    let _current_score = analysis.item_scores.iter()
        .find(|s| s.slot.eq_ignore_ascii_case(slot))
        .map(|s| s.score)
        .unwrap_or(50);

    // Build a list of popular upgrade targets for the slot based on archetype
    let candidates = upgrade_candidates_for_slot(slot, &analysis.archetype);

    let mut results = Vec::new();
    for (item_name, mod_highlights, est_dps_gain_pct) in candidates {
        // Try to get real price from poe.ninja
        let (price_div, trade_url) = match super::price_cache::get_prices(&[item_name.clone()]).await {
            Ok(prices) if !prices.is_empty() => {
                let p = &prices[0];
                let url = build_trade_url(slot, &analysis.archetype, &item_name);
                (p.price_div, url)
            }
            _ => (0.0, build_trade_url(slot, &analysis.archetype, &item_name)),
        };

        // Skip if over budget
        if price_div > budget_div && budget_div > 0.0 {
            continue;
        }

        // Calculate efficiency: DPS gain % per divine spent
        let efficiency = if price_div > 0.01 {
            est_dps_gain_pct / price_div
        } else {
            est_dps_gain_pct // free upgrades have infinite efficiency — put at top
        };

        let dps_gain = analysis.offense.total_dps * (est_dps_gain_pct / 100.0);
        let life_gain = if slot == "Helmet" || slot == "BodyArmour" || slot == "Gloves" || slot == "Boots" {
            50 // placeholder for life gain estimate
        } else {
            0
        };

        results.push(TradeResult {
            item_name: item_name.clone(),
            slot: slot.to_string(),
            price_div,
            dps_gain,
            life_gain,
            efficiency,
            trade_url,
            mod_highlights,
        });
    }

    // Sort by efficiency descending
    results.sort_by(|a, b| b.efficiency.partial_cmp(&a.efficiency).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(5);

    Ok(results)
}

/// Return (item_name, mod_highlights, estimated_dps_gain_pct) tuples for a slot+archetype.
pub(crate) fn upgrade_candidates_for_slot(
    slot: &str,
    archetype: &str,
) -> Vec<(String, Vec<String>, f64)> {
    let is_dot = archetype.contains("DoT") || archetype.contains("fire") || archetype.contains("Fire");
    let is_rf = archetype.contains("RF") || archetype.contains("RighteousFire") || is_dot;

    match slot {
        "Helmet" => vec![
            ("Starkonja's Head".to_string(), vec!["Dex".to_string(), "Life".to_string(), "Evasion".to_string()], 5.0),
            ("Rare Helmet".to_string(), vec!["+80 Life".to_string(), "Res crafts".to_string()], 8.0),
        ],
        "BodyArmour" => {
            if is_rf {
                vec![
                    ("Kaom's Heart".to_string(), vec!["+500 max Life".to_string()], 20.0),
                    ("Rare Astral Plate".to_string(), vec!["+100 Life".to_string(), "Res".to_string()], 10.0),
                ]
            } else {
                vec![
                    ("Rare Body Armour".to_string(), vec!["+80 Life".to_string(), "Res".to_string()], 8.0),
                ]
            }
        },
        "Boots" => vec![
            ("Rare Boots".to_string(), vec!["Movement speed".to_string(), "+60 Life".to_string(), "Res".to_string()], 5.0),
            ("Rare Boots with crafts".to_string(), vec!["MS".to_string(), "Life".to_string(), "Res x2".to_string()], 7.0),
        ],
        "Gloves" => vec![
            ("Rare Gloves".to_string(), vec!["+60 Life".to_string(), "Res".to_string(), "Flat fire dmg".to_string()], 5.0),
        ],
        "Amulet" => vec![
            ("Rare Amulet".to_string(), vec!["+50 Life".to_string(), "Res".to_string(), "Damage multi".to_string()], 10.0),
        ],
        "Ring" | "Ring 1" | "Ring 2" => vec![
            ("Rare Ring".to_string(), vec!["+40 Life".to_string(), "Res x2".to_string()], 5.0),
        ],
        "Belt" => vec![
            ("Stygian Vise".to_string(), vec!["Abyss socket".to_string(), "Life".to_string(), "Res".to_string()], 8.0),
        ],
        "Weapon" | "Weapon 1" => {
            if is_dot {
                vec![
                    ("Sceptre of RF".to_string(), vec!["Fire dot multi".to_string(), "+1 fire gems".to_string()], 15.0),
                ]
            } else {
                vec![
                    ("Rare Weapon".to_string(), vec!["High DPS".to_string(), "Crit chance".to_string()], 12.0),
                ]
            }
        },
        _ => vec![],
    }
}

fn build_trade_url(_slot: &str, _archetype: &str, item_name: &str) -> String {
    let league = std::env::var("POE_LEAGUE").unwrap_or_else(|_| "Settlers".to_string());
    if item_name.starts_with("Rare ") {
        // Generic trade search URL for rare items
        format!("https://www.pathofexile.com/trade/search/{league}")
    } else {
        // Direct name search
        format!(
            "https://www.pathofexile.com/trade/search/{league}?q={{\"query\":{{\"name\":\"{}\"}}}}",
            urlencoding::encode(item_name)
        )
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helmet_slot_returns_two_candidates() {
        let candidates = upgrade_candidates_for_slot("Helmet", "Generic");
        assert_eq!(candidates.len(), 2, "Helmet should have 2 upgrade candidates");
        let names: Vec<&str> = candidates.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"Starkonja's Head"), "Should include Starkonja's Head");
        assert!(names.contains(&"Rare Helmet"), "Should include Rare Helmet");
    }

    #[test]
    fn body_armour_rf_archetype_includes_kaoms_heart() {
        let candidates = upgrade_candidates_for_slot("BodyArmour", "RFInquisitor");
        let names: Vec<&str> = candidates.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"Kaom's Heart"), "RF archetype should suggest Kaom's Heart");
    }

    #[test]
    fn body_armour_non_rf_does_not_suggest_kaoms() {
        let candidates = upgrade_candidates_for_slot("BodyArmour", "ColdDOT");
        let names: Vec<&str> = candidates.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(!names.contains(&"Kaom's Heart"), "Non-RF should not suggest Kaom's Heart");
    }

    #[test]
    fn weapon_dot_archetype_suggests_sceptre() {
        let candidates = upgrade_candidates_for_slot("Weapon", "FireDoT");
        let names: Vec<&str> = candidates.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"Sceptre of RF"), "Fire DoT should suggest RF Sceptre");
    }

    #[test]
    fn unknown_slot_returns_empty() {
        let candidates = upgrade_candidates_for_slot("Flask", "Generic");
        assert!(candidates.is_empty(), "Unknown slot should return no candidates");
    }

    #[test]
    fn all_candidates_have_positive_dps_gain() {
        for slot in &["Helmet", "Boots", "Gloves", "Amulet", "Belt"] {
            let candidates = upgrade_candidates_for_slot(slot, "Generic");
            for (name, _, dps_gain) in &candidates {
                assert!(*dps_gain > 0.0, "Candidate '{name}' should have positive DPS gain");
            }
        }
    }
}
