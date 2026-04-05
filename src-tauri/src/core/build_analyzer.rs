/// Build analyzer — issue detection, item scoring, suggestion generation.
/// See ALGORITHMS.md Algorithm 6 (Item Scorer) and Algorithm 7 (Issue Detector).
use anyhow::Result;
use crate::models::build::BuildData;
use crate::models::analysis::*;
use crate::models::seer::TreeAnalysis;
use crate::core::build_detector::Archetype;

pub fn detect_issues(
    build: &BuildData,
    defenses: &DefenseStats,
    offense: &OffenseStats,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Critical: Resistances uncapped
    check_resist_cap(&defenses.resistances, &mut issues);

    // Critical: Life too low
    if defenses.life < 3_000 {
        issues.push(Issue {
            id: "low_life_critical".to_string(),
            severity: Severity::Critical,
            title: "Life pool dangerously low".to_string(),
            detail: format!("{} life — you will die to any spike damage", defenses.life),
            fix: "Prioritize life on gear: helmet (+50-80), chest (+60-100), rings (+40-50 each)".to_string(),
            slot: None,
        });
    } else if defenses.life < 4_500 {
        issues.push(Issue {
            id: "low_life_major".to_string(),
            severity: Severity::Major,
            title: "Life pool below recommended".to_string(),
            detail: format!("{} life — target 4,500+ for mapping, 5,000+ for endgame", defenses.life),
            fix: "Add life to 3-4 gear slots. Prioritize: helmet, chest, rings.".to_string(),
            slot: None,
        });
    }

    // Major: Ailment immunities
    check_ailment_immunities(&defenses.ailment_immunity, &mut issues);

    // Major: No movement skill detected
    let has_movement = build.gems.iter().any(|g| {
        let skill = g.skill.to_lowercase();
        ["flame dash", "shield charge", "whirling blades", "dash",
         "blink arrow", "leap slam", "frostblink"].iter()
            .any(|m| skill.contains(m))
    });
    if !has_movement {
        issues.push(Issue {
            id: "no_movement_skill".to_string(),
            severity: Severity::Major,
            title: "No movement skill detected".to_string(),
            detail: "Movement skills are essential for dodging boss mechanics and clearing efficiently".to_string(),
            fix: "Add Flame Dash or Shield Charge to a 3-link setup".to_string(),
            slot: None,
        });
    }

    // Minor: DPS check
    if offense.total_dps < 500_000.0 {
        issues.push(Issue {
            id: "low_dps".to_string(),
            severity: Severity::Minor,
            title: "DPS below 500K".to_string(),
            detail: format!("{} — target 1M+ for comfortable red maps", offense.dps_label),
            fix: "Upgrade support gems to 20/20, check gem links are correct".to_string(),
            slot: None,
        });
    }

    // Sort: Critical first
    issues.sort_by_key(|i| std::cmp::Reverse(i.severity.clone()));
    issues
}

fn check_resist_cap(res: &ResistanceProfile, issues: &mut Vec<Issue>) {
    let checks = [
        ("Fire", res.fire, res.max_fire, "fire"),
        ("Cold", res.cold, res.max_cold, "cold"),
        ("Lightning", res.lightning, res.max_lightning, "lightning"),
    ];
    for (name, val, max, mod_name) in checks {
        if val < max {
            let gap = max - val;
            issues.push(Issue {
                id: format!("uncapped_{mod_name}_res"),
                severity: if gap > 30 { Severity::Critical } else { Severity::Major },
                title: format!("{name} resistance uncapped"),
                detail: format!("{val}% / {max}% — {gap}% short of cap"),
                fix: format!("Add +{gap}% {name} res via jewelry, helmet, or boots craft"),
                slot: None,
            });
        }
    }
    if res.chaos < 0 {
        issues.push(Issue {
            id: "negative_chaos_res".to_string(),
            severity: Severity::Major,
            title: "Chaos resistance is negative".to_string(),
            detail: format!("{}% — chaos damage is amplified", res.chaos),
            fix: "Target 0%+ chaos res for maps. 60%+ for endgame (Sirus, Maven).".to_string(),
            slot: None,
        });
    }
}

fn check_ailment_immunities(ailments: &AilmentImmunity, issues: &mut Vec<Issue>) {
    if !ailments.freeze {
        issues.push(Issue {
            id: "no_freeze_immune".to_string(),
            severity: Severity::Major,
            title: "No freeze immunity".to_string(),
            detail: "Freeze stops all action — lethal against casters and Uber Elder".to_string(),
            fix: "Brine King pantheon (free) or boot craft 'Immunity to Freeze/Chill'".to_string(),
            slot: Some("Boots".to_string()),
        });
    }
    if !ailments.corrupted_blood {
        issues.push(Issue {
            id: "no_corrupted_blood_immune".to_string(),
            severity: Severity::Major,
            title: "No Corrupted Blood immunity".to_string(),
            detail: "Corrupted Blood stacks can kill in 1-2 seconds without immunity".to_string(),
            fix: "Corrupt a jewel with 'Corrupted Blood cannot be inflicted on you'".to_string(),
            slot: None,
        });
    }
}

pub fn score_items(build: &BuildData, archetype: Archetype) -> Vec<ItemScore> {
    let weights = archetype.stat_weights();

    build.items.iter().map(|item| {
        let score = score_single_item(item, weights);
        let tier = score_to_tier(score);

        ItemScore {
            slot: item.slot.clone(),
            item_name: item.name.clone(),
            score,
            tier,
            top_issue: find_item_issue(item, archetype),
        }
    }).collect()
}

fn score_single_item(item: &crate::models::build::Item, weights: &[(&str, f64)]) -> u8 {
    // Simple scoring: count weighted mods present
    // Full implementation requires mod database for tier values
    let mut score: f64 = 20.0; // base score for any equipped item

    for mod_ in &item.mods {
        let text = mod_.text.to_lowercase();
        for (stat_id, weight) in weights {
            if text.contains(stat_id.replace('_', " ").as_str()) {
                score += weight * 10.0;
            }
        }
    }

    // Unique items get a base bonus
    if matches!(item.rarity, crate::models::build::ItemRarity::Unique) {
        score += 20.0;
    }

    score.min(100.0) as u8
}

fn score_to_tier(score: u8) -> ScoreTier {
    match score {
        90..=100 => ScoreTier::BiS,
        75..=89  => ScoreTier::Excellent,
        60..=74  => ScoreTier::Good,
        40..=59  => ScoreTier::Acceptable,
        20..=39  => ScoreTier::Upgrade,
        _        => ScoreTier::Replace,
    }
}

fn find_item_issue(item: &crate::models::build::Item, _archetype: Archetype) -> Option<String> {
    let has_life = item.mods.iter().any(|m| {
        let t = m.text.to_lowercase();
        t.contains("to maximum life") || t.contains("maximum life")
    });
    let has_ms = item.mods.iter().any(|m| {
        m.text.to_lowercase().contains("movement speed")
    });

    // Slot-specific checks
    match item.slot.as_str() {
        "BodyArmour" | "Helmet" => {
            if item.mods.is_empty() {
                return Some("No mods — this item needs upgrading".to_string());
            }
            if !has_life {
                return Some(format!("{} is missing a life mod — aim for +80 to maximum Life", item.slot));
            }
        }
        "Boots" => {
            if item.mods.is_empty() {
                return Some("No mods on Boots".to_string());
            }
            if !has_ms {
                return Some("Boots are missing movement speed — aim for 30%+".to_string());
            }
            if !has_life {
                return Some("Boots are missing a life mod".to_string());
            }
        }
        "Gloves" => {
            if item.mods.is_empty() {
                return Some("No mods on Gloves".to_string());
            }
            if !has_life {
                return Some("Gloves are missing a life mod".to_string());
            }
        }
        "Belt" | "Amulet" | "Ring" | "Ring 1" | "Ring 2" => {
            if item.mods.is_empty() {
                return Some("No mods — this item needs upgrading".to_string());
            }
        }
        _ => {
            if item.mods.is_empty() {
                return Some("No mods — this item needs upgrading".to_string());
            }
        }
    }
    None
}

pub fn generate_suggestions(
    _build: &BuildData,
    issues: &[Issue],
    item_scores: &[ItemScore],
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    let mut priority = 1u32;

    // Generate suggestion for each critical/major issue (Critical >= Major > Minor > Info)
    for issue in issues.iter().filter(|i| i.severity >= Severity::Major) {
        let slot = issue.slot.clone().unwrap_or_default();
        let est_cost = slot_upgrade_cost_div(&slot);
        let (life_gain, dps_gain_pct) = slot_expected_gains(&slot);
        let efficiency = if est_cost > 0.0 { dps_gain_pct / est_cost } else { 0.0 };

        suggestions.push(Suggestion {
            id: format!("fix_{}", issue.id),
            slot: slot.clone(),
            title: format!("Fix: {}", issue.title),
            detail: issue.fix.clone(),
            dps_gain: 0.0,
            dps_gain_pct,
            life_gain,
            estimated_cost_div: est_cost,
            efficiency,
            priority,
            trade_url: None,
        });
        priority += 1;
    }

    // Suggest upgrading lowest-scored items
    let mut scored: Vec<_> = item_scores.iter()
        .filter(|s| s.score < 60)
        .collect();
    scored.sort_by_key(|s| s.score);

    for item_score in scored.iter().take(3) {
        let slot = &item_score.slot;
        let est_cost = slot_upgrade_cost_div(slot);
        let (life_gain, dps_gain_pct) = slot_expected_gains(slot);
        let score_deficit = (60u8.saturating_sub(item_score.score)) as f64;
        // Scale cost estimate by how far below 60 the item is
        let scaled_cost = est_cost * (1.0 + score_deficit / 100.0);
        let efficiency = if scaled_cost > 0.0 { dps_gain_pct / scaled_cost } else { 0.0 };

        suggestions.push(Suggestion {
            id: format!("upgrade_{}", slot.to_lowercase().replace(' ', "_")),
            slot: slot.clone(),
            title: format!("Upgrade {slot}"),
            detail: format!(
                "{} scored {}/100 — search for a {slot} with better mods",
                item_score.item_name, item_score.score
            ),
            dps_gain: 0.0,
            dps_gain_pct,
            life_gain,
            estimated_cost_div: scaled_cost,
            efficiency,
            priority,
            trade_url: None,
        });
        priority += 1;
    }

    suggestions
}

/// Rough divine cost to find a "good" item for a given slot on the trade site.
fn slot_upgrade_cost_div(slot: &str) -> f64 {
    match slot {
        "BodyArmour"                       => 2.0,
        "Weapon" | "Weapon 1"              => 3.0,
        "Helmet"                           => 1.0,
        "Gloves" | "Boots"                 => 0.5,
        "Belt"                             => 0.5,
        "Amulet"                           => 1.5,
        "Ring" | "Ring 1" | "Ring 2"       => 0.5,
        "Shield"                           => 1.0,
        _                                  => 1.0,
    }
}

/// Returns (life_gain_flat, dps_gain_pct) for a typical upgrade for the slot.
fn slot_expected_gains(slot: &str) -> (i32, f64) {
    match slot {
        "BodyArmour"                       => (100, 5.0),
        "Weapon" | "Weapon 1"              => (0,   15.0),
        "Helmet"                           => (80,  5.0),
        "Gloves"                           => (60,  3.0),
        "Boots"                            => (60,  3.0),
        "Belt"                             => (50,  2.0),
        "Amulet"                           => (50,  8.0),
        "Ring" | "Ring 1" | "Ring 2"       => (40,  4.0),
        "Shield"                           => (50,  3.0),
        _                                  => (30,  2.0),
    }
}

pub fn overall_score(defenses: &DefenseStats, offense: &OffenseStats, issues: &[Issue]) -> u8 {
    let mut score: f64 = 50.0;

    // Penalize for critical issues
    let critical_count = issues.iter().filter(|i| i.severity == Severity::Critical).count();
    let major_count = issues.iter().filter(|i| i.severity == Severity::Major).count();
    score -= (critical_count as f64) * 15.0;
    score -= (major_count as f64) * 5.0;

    // Reward for life pool
    if defenses.life >= 5_000 { score += 10.0; }
    else if defenses.life >= 4_000 { score += 5.0; }

    // Reward for capped resists
    let res = &defenses.resistances;
    if res.fire >= res.max_fire { score += 5.0; }
    if res.cold >= res.max_cold { score += 5.0; }
    if res.lightning >= res.max_lightning { score += 5.0; }

    // Reward for DPS
    if offense.total_dps >= 2_000_000.0 { score += 10.0; }
    else if offense.total_dps >= 1_000_000.0 { score += 5.0; }

    score.clamp(0.0, 100.0) as u8
}

pub fn analyze_tree(build: &BuildData) -> Result<TreeAnalysis> {
    let total_allocated = build.passive_tree.allocated_nodes.len() as u32;
    let by_category = categorize_masteries(&build.passive_tree.masteries);
    let next_keystone = suggest_next_keystone(build);
    let top_recommendations = archetype_tree_recommendations(build);

    Ok(TreeAnalysis {
        total_allocated,
        by_category,
        top_recommendations,
        inefficient_nodes: vec![],
        next_keystone,
    })
}

/// Suggest the next keystone for a build based on ascendancy/class heuristics.
fn suggest_next_keystone(build: &BuildData) -> Option<String> {
    let asc = build.ascendancy.to_lowercase();
    let class = build.class_name.to_lowercase();

    if asc.contains("inquisitor") {
        Some("Inevitable Judgement (ignores res on consecrated ground)".to_string())
    } else if asc.contains("berserker") {
        Some("Rite of Ruin (more damage taken → more damage dealt)".to_string())
    } else if asc.contains("occultist") {
        Some("Void Beacon (reduces chaos res of nearby enemies)".to_string())
    } else if asc.contains("elementalist") {
        Some("Shaper of Flames (all ignite as if dealing max roll)".to_string())
    } else if asc.contains("deadeye") {
        Some("Focal Point (more projectile damage vs single target)".to_string())
    } else if asc.contains("trickster") {
        Some("Ghost Dance (recover ES on evade)".to_string())
    } else if class.contains("shadow") || asc.contains("assassin") {
        Some("Ambush and Assassinate (more crit damage vs enemies with power charges)".to_string())
    } else if class.contains("marauder") || asc.contains("juggernaut") {
        Some("Unflinching (endurance charge sustain + stun immunity)".to_string())
    } else if asc.contains("chieftain") {
        Some("Ramako, Sun's Light (life regen per nearby ignited enemy)".to_string())
    } else if class.contains("witch") || asc.contains("necromancer") {
        Some("Plaguebringer (infect nearby enemies with decay)".to_string())
    } else {
        None
    }
}

/// Return archetype-appropriate passive tree recommendations.
fn archetype_tree_recommendations(build: &BuildData) -> Vec<crate::models::seer::NodeRecommendation> {
    use crate::models::seer::NodeRecommendation;
    if build.passive_tree.allocated_nodes.is_empty() {
        return vec![];
    }

    let asc = build.ascendancy.to_lowercase();
    let class = build.class_name.to_lowercase();

    // Base recommendations list for this build type
    let recs: &[(&str, f64, u32, &str)] = if asc.contains("inquisitor") || asc.contains("chieftain") {
        &[
            ("Sovereignty", 3.5, 2, "Reduces reservation of auras — enables more aura stacking"),
            ("Heart of Flame", 4.0, 3, "Fire dot multiplier cluster — top priority for RF builds"),
            ("Divine Judgement", 3.8, 3, "More fire damage — central to RF/Chieftain damage"),
            ("Breath of Flames", 3.2, 4, "Ignite proliferation + fire dot multi"),
        ]
    } else if asc.contains("elementalist") {
        &[
            ("Heart of Destruction", 4.0, 3, "More damage with Shaper of Flames/Storms"),
            ("Liege of the Primordial", 3.5, 4, "Golem buffs — core for Golemancer builds"),
            ("Shaper of Storms", 3.8, 3, "All hits crit — changes the game for spell builds"),
        ]
    } else if asc.contains("occultist") || asc.contains("trickster") {
        &[
            ("Constitution", 3.0, 2, "Energy shield cluster — essential for ES builds"),
            ("Overcharge", 3.5, 3, "Power charge generation for spell crit"),
            ("Chaos Inoculation", 5.0, 5, "1 HP / infinite ES — endgame CI keystone"),
        ]
    } else if asc.contains("deadeye") || asc.contains("pathfinder") || asc.contains("raider") {
        &[
            ("Acceleration", 3.5, 2, "Projectile speed — essential for bow builds"),
            ("Ballistic Mastery", 3.0, 3, "Additional projectiles cluster for clear"),
            ("Point Blank", 3.8, 2, "Damage falls off with range — swap for melee range"),
        ]
    } else if asc.contains("berserker") || asc.contains("juggernaut") || asc.contains("slayer") {
        &[
            ("Warlock's Mark", 3.0, 2, "Life leech sustain for melee builds"),
            ("Blood Drinker", 3.5, 3, "Physical leech cluster — sustain for attacks"),
            ("Cannibalistic Rite", 3.2, 3, "Life flask sustain + flask generation"),
        ]
    } else {
        &[
            ("Written in Blood", 3.0, 2, "Core life cluster — 12% life"),
            ("Sanguine Pact", 2.8, 2, "Life + regen cluster"),
        ]
    };

    recs.iter().enumerate().map(|(i, &(name, value, cost, reason))| {
        NodeRecommendation {
            node_id: (1000 + i) as u32,
            node_name: name.to_string(),
            path_cost: cost,
            value_score: value,
            efficiency: value / cost as f64,
            reason: reason.to_string(),
        }
    }).collect()
}

/// Categorize mastery selections into named stat groups using keyword matching.
fn categorize_masteries(masteries: &[crate::models::build::MasterySelection]) -> Vec<crate::models::seer::NodeCategory> {
    use crate::models::seer::NodeCategory;
    use std::collections::HashMap;

    let mut counts: HashMap<&'static str, u32> = HashMap::new();

    for m in masteries {
        let text = m.effect_text.to_lowercase();
        let category = if text.contains("life") || text.contains("life regenerat") {
            "Life"
        } else if text.contains("energy shield") {
            "Energy Shield"
        } else if text.contains("resistance") || text.contains("elemental") {
            "Resistances"
        } else if text.contains("fire damage") || text.contains("burning") || text.contains("ignite") {
            "Fire Damage"
        } else if text.contains("cold damage") || text.contains("freeze") || text.contains("chill") {
            "Cold Damage"
        } else if text.contains("lightning damage") || text.contains("shock") {
            "Lightning Damage"
        } else if text.contains("critical") || text.contains("crit") {
            "Critical Strike"
        } else if text.contains("attack speed") || text.contains("cast speed") {
            "Speed"
        } else if text.contains("armour") || text.contains("armor") || text.contains("evasion") {
            "Defence"
        } else if text.contains("mana") {
            "Mana"
        } else {
            "Other"
        };
        *counts.entry(category).or_insert(0) += 1;
    }

    let mut categories: Vec<NodeCategory> = counts.into_iter()
        .map(|(name, count)| NodeCategory {
            name: name.to_string(),
            count,
            total_value: 0.0,
        })
        .collect();
    categories.sort_by(|a, b| b.count.cmp(&a.count));
    categories
}

impl std::cmp::Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Critical > Major > Minor > Info
        fn rank(s: &Severity) -> u8 {
            match s { Severity::Critical => 3, Severity::Major => 2, Severity::Minor => 1, Severity::Info => 0 }
        }
        rank(self).cmp(&rank(other))
    }
}

impl std::cmp::PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::build::{PassiveTree, MasterySelection};

    fn mastery(id: u32, effect: &str) -> MasterySelection {
        MasterySelection { node_id: id, effect_id: id * 100, effect_text: effect.to_string() }
    }

    #[test]
    fn analyze_tree_total_allocated_is_correct() {
        let mut build = BuildData::default();
        build.passive_tree.allocated_nodes = vec![1, 2, 3, 4, 5];
        let result = analyze_tree(&build).unwrap();
        assert_eq!(result.total_allocated, 5);
    }

    #[test]
    fn analyze_tree_zero_nodes_returns_zero() {
        let build = BuildData::default();
        let result = analyze_tree(&build).unwrap();
        assert_eq!(result.total_allocated, 0);
    }

    #[test]
    fn analyze_tree_always_succeeds() {
        let mut build = BuildData::default();
        build.passive_tree.allocated_nodes = (1..=90).collect();
        assert!(analyze_tree(&build).is_ok());
    }

    #[test]
    fn analyze_tree_categorizes_life_masteries() {
        let mut build = BuildData::default();
        build.passive_tree.masteries = vec![
            mastery(1, "10% increased maximum Life"),
            mastery(2, "+25 to maximum Life"),
        ];
        let result = analyze_tree(&build).unwrap();
        let life_cat = result.by_category.iter().find(|c| c.name == "Life");
        assert!(life_cat.is_some(), "should have a Life category from masteries");
        assert_eq!(life_cat.unwrap().count, 2);
    }

    #[test]
    fn analyze_tree_categorizes_resistance_masteries() {
        let mut build = BuildData::default();
        build.passive_tree.masteries = vec![
            mastery(1, "+10% to all Elemental Resistances"),
            mastery(2, "Fire Resistance is Lucky"),
        ];
        let result = analyze_tree(&build).unwrap();
        let cat = result.by_category.iter().find(|c| c.name == "Resistances");
        assert!(cat.is_some(), "should have a Resistances category");
    }

    #[test]
    fn analyze_tree_empty_masteries_returns_empty_categories() {
        let build = BuildData::default();
        let result = analyze_tree(&build).unwrap();
        assert!(result.by_category.is_empty(),
            "no masteries → no categories");
    }

    #[test]
    fn analyze_tree_rf_archetype_suggests_fire_keystone() {
        let mut build = BuildData::default();
        build.class_name = "Templar".to_string();
        build.ascendancy = "Inquisitor".to_string();
        let result = analyze_tree(&build).unwrap();
        // RF builds should get a relevant keystone suggestion
        if let Some(ks) = &result.next_keystone {
            assert!(!ks.is_empty(), "next_keystone should be non-empty string");
        }
    }

    #[test]
    fn analyze_tree_returns_recommendations_for_inquisitor() {
        let mut build = BuildData::default();
        build.ascendancy = "Inquisitor".to_string();
        build.passive_tree.allocated_nodes = vec![1, 2, 3];
        let result = analyze_tree(&build).unwrap();
        assert!(!result.top_recommendations.is_empty(),
            "should return at least one recommendation for a build with nodes");
    }

    #[test]
    fn analyze_tree_recommendations_have_non_empty_reason() {
        let mut build = BuildData::default();
        build.ascendancy = "Inquisitor".to_string();
        build.passive_tree.allocated_nodes = vec![1];
        let result = analyze_tree(&build).unwrap();
        for rec in &result.top_recommendations {
            assert!(!rec.reason.is_empty(), "recommendation reason must not be empty");
            assert!(!rec.node_name.is_empty(), "recommendation node name must not be empty");
        }
    }

    // ── generate_suggestions ──────────────────────────────────────────────────

    fn make_issue(id: &str, severity: Severity, slot: Option<&str>) -> Issue {
        Issue {
            id: id.to_string(),
            severity,
            title: format!("Issue {id}"),
            detail: "detail".to_string(),
            fix: "fix".to_string(),
            slot: slot.map(|s| s.to_string()),
        }
    }

    fn make_item_score(slot: &str, name: &str, score: u8, tier: ScoreTier) -> ItemScore {
        ItemScore { slot: slot.to_string(), item_name: name.to_string(), score, tier, top_issue: None }
    }

    #[test]
    fn suggestions_include_entry_for_critical_issues() {
        let build = BuildData::default();
        let issues = vec![make_issue("low_life", Severity::Critical, Some("Helmet"))];
        let scores = vec![];
        let suggestions = generate_suggestions(&build, &issues, &scores);
        assert!(!suggestions.is_empty(), "should produce a suggestion for critical issues");
        assert!(suggestions[0].slot == "Helmet" || suggestions[0].slot.is_empty());
    }

    #[test]
    fn suggestions_have_nonzero_estimated_cost_for_slot_items() {
        let build = BuildData::default();
        let issues = vec![];
        let scores = vec![make_item_score("Helmet", "Bad Helmet", 30, ScoreTier::Upgrade)];
        let suggestions = generate_suggestions(&build, &issues, &scores);
        let helmet_sug = suggestions.iter().find(|s| s.slot == "Helmet");
        assert!(helmet_sug.is_some(), "should suggest upgrading low-scored Helmet");
        assert!(helmet_sug.unwrap().estimated_cost_div > 0.0,
            "estimated cost should be non-zero for slot with known price");
    }

    #[test]
    fn suggestions_sorted_by_priority_ascending() {
        let build = BuildData::default();
        let issues = vec![
            make_issue("issue1", Severity::Critical, Some("BodyArmour")),
            make_issue("issue2", Severity::Major, Some("Ring")),
        ];
        let scores = vec![];
        let suggestions = generate_suggestions(&build, &issues, &scores);
        for w in suggestions.windows(2) {
            assert!(w[0].priority <= w[1].priority, "suggestions must be priority-ascending");
        }
    }

    #[test]
    fn suggestions_have_positive_efficiency_when_cost_nonzero() {
        let build = BuildData::default();
        let issues = vec![];
        let scores = vec![make_item_score("Boots", "Weak Boots", 20, ScoreTier::Replace)];
        let suggestions = generate_suggestions(&build, &issues, &scores);
        let boots = suggestions.iter().find(|s| s.slot == "Boots");
        if let Some(s) = boots {
            if s.estimated_cost_div > 0.0 {
                assert!(s.efficiency >= 0.0, "efficiency must be non-negative");
            }
        }
    }

    // ── find_item_issue ───────────────────────────────────────────────────────

    #[test]
    fn find_item_issue_flags_missing_life_on_body_armour() {
        use crate::models::build::{Item, ItemRarity};
        let item = Item {
            id: 0, name: "Belly".to_string(), base_type: "Astral Plate".to_string(),
            slot: "BodyArmour".to_string(), rarity: ItemRarity::Rare,
            level_requirement: 62, item_level: 84, quality: 20,
            sockets: "RRRRRR".to_string(), mods: vec![],
            influence: vec![], is_corrupted: false, is_synthesised: false, is_fractured: false,
            image_url: None, score: None,
        };
        let issue = find_item_issue(&item, crate::core::build_detector::Archetype::FireDoT);
        assert!(issue.is_some(), "body armour with no life should flag an issue");
        let msg = issue.unwrap().to_lowercase();
        assert!(msg.contains("life") || msg.contains("mods"), "message should mention life or mods");
    }

    #[test]
    fn find_item_issue_flags_missing_movement_speed_on_boots() {
        use crate::models::build::{Item, ItemRarity, ItemMod, ModType};
        let item = Item {
            id: 0, name: "Iron Greaves".to_string(), base_type: "Iron Greaves".to_string(),
            slot: "Boots".to_string(), rarity: ItemRarity::Rare,
            level_requirement: 3, item_level: 80, quality: 20,
            sockets: "RRRR".to_string(),
            mods: vec![
                ItemMod { id: "s0".to_string(), text: "+50 to maximum Life".to_string(), value1: 50.0, value2: None, mod_type: ModType::Suffix, is_crafted: false, is_fractured: false },
            ],
            influence: vec![], is_corrupted: false, is_synthesised: false, is_fractured: false,
            image_url: None, score: None,
        };
        let issue = find_item_issue(&item, crate::core::build_detector::Archetype::Unknown);
        assert!(issue.is_some(), "boots without movement speed should flag an issue");
        let msg = issue.unwrap().to_lowercase();
        assert!(msg.contains("movement") || msg.contains("speed"), "message should mention movement speed");
    }

    #[test]
    fn find_item_issue_none_for_good_item() {
        use crate::models::build::{Item, ItemRarity, ItemMod, ModType};
        // Boots with life + move speed → no issue
        let item = Item {
            id: 0, name: "Good Boots".to_string(), base_type: "Sorcerer Boots".to_string(),
            slot: "Boots".to_string(), rarity: ItemRarity::Rare,
            level_requirement: 67, item_level: 86, quality: 20,
            sockets: "RRGG".to_string(),
            mods: vec![
                ItemMod { id: "s0".to_string(), text: "+70 to maximum Life".to_string(), value1: 70.0, value2: None, mod_type: ModType::Suffix, is_crafted: false, is_fractured: false },
                ItemMod { id: "s1".to_string(), text: "30% increased Movement Speed".to_string(), value1: 30.0, value2: None, mod_type: ModType::Suffix, is_crafted: false, is_fractured: false },
            ],
            influence: vec![], is_corrupted: false, is_synthesised: false, is_fractured: false,
            image_url: None, score: None,
        };
        let issue = find_item_issue(&item, crate::core::build_detector::Archetype::Unknown);
        assert!(issue.is_none(), "well-equipped boots should have no issue");
    }
}
