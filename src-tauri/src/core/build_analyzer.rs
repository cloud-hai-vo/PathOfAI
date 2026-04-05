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
    if item.mods.is_empty() {
        return Some("No mods — this item needs upgrading".to_string());
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

    // Generate suggestion for each critical/major issue
    for issue in issues.iter().filter(|i| i.severity <= Severity::Major) {
        suggestions.push(Suggestion {
            id: format!("fix_{}", issue.id),
            slot: issue.slot.clone().unwrap_or_default(),
            title: format!("Fix: {}", issue.title),
            detail: issue.fix.clone(),
            dps_gain: 0.0,
            dps_gain_pct: 0.0,
            life_gain: 0,
            estimated_cost_div: 0.0, // populated by market module
            efficiency: 0.0,
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
        suggestions.push(Suggestion {
            id: format!("upgrade_{}", item_score.slot.to_lowercase()),
            slot: item_score.slot.clone(),
            title: format!("Upgrade {}", item_score.slot),
            detail: format!("{} scored {}/100 — search for better options", item_score.item_name, item_score.score),
            dps_gain: 0.0,
            dps_gain_pct: 0.0,
            life_gain: 0,
            estimated_cost_div: 0.0,
            efficiency: 0.0,
            priority,
            trade_url: None,
        });
        priority += 1;
    }

    suggestions
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
    Ok(TreeAnalysis {
        total_allocated: build.passive_tree.allocated_nodes.len() as u32,
        by_category: vec![],       // TODO: requires tree data
        top_recommendations: vec![],
        inefficient_nodes: vec![],
        next_keystone: None,
    })
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
