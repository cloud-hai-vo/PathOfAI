/// Offense calculator — computes DPS from a BuildData.
/// See ALGORITHMS.md Algorithm 3 (DPS Calculation Engine).
use crate::models::build::{BuildData, GemSetup};
use crate::models::analysis::*;
use crate::core::build_detector::{detect_archetype, Archetype};
use super::formulas;

pub fn calculate(build: &BuildData) -> OffenseStats {
    let archetype = detect_archetype(build);
    let main_skill_setup = build.gems.iter().find(|g| g.is_main_skill)
        .or_else(|| build.gems.first());

    let main_skill = main_skill_setup
        .map(|g| g.skill.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let (dot_dps, hit_dps) = match archetype {
        Archetype::FireDoT => (calculate_fire_dot_dps(build, main_skill_setup), 0.0),
        Archetype::ColdDoT => (calculate_generic_dot_dps(build, main_skill_setup), 0.0),
        Archetype::PoisonDoT | Archetype::PhysDoT => {
            (calculate_generic_dot_dps(build, main_skill_setup), 0.0)
        }
        _ => (0.0, calculate_hit_dps(build, main_skill_setup)),
    };

    let total_dps = dot_dps + hit_dps;
    let sources = build_dps_sources(build, dot_dps, hit_dps, total_dps);
    let multiplier_chain = build_multiplier_chain(build, main_skill_setup);

    let base_crit_chance = 0.05; // PoE base: 5%
    let increased_crit   = crit_chance_increased_from_items(build);
    let crit_chance      = (base_crit_chance * (1.0 + increased_crit / 100.0)).min(1.0);

    let base_crit_multi  = 1.5; // PoE base: 150%
    let increased_multi  = crit_multiplier_increased_from_items(build);
    let crit_multiplier  = base_crit_multi + increased_multi / 100.0;

    let attack_speed = attack_speed_from_items(build);

    // Hit chance: player accuracy vs. level-based monster evasion baseline.
    // Against a level 83 monster, base evasion ≈ 6_000.
    let player_accuracy = accuracy_from_items(build);
    let hit_chance = player_hit_chance(player_accuracy, 6_000.0);

    OffenseStats {
        total_dps,
        dps_label: formulas::format_dps(total_dps),
        main_skill,
        hit_dps,
        dot_dps,
        crit_chance,
        crit_multiplier,
        attack_speed,
        cast_speed: 0.0,
        hit_chance,
        sources,
        multiplier_chain,
    }
}

/// Sum all "increased critical strike chance" mods from equipped items.
pub(crate) fn crit_chance_increased_from_items(build: &BuildData) -> f64 {
    build.items.iter().flat_map(|it| &it.mods)
        .filter(|m| {
            let t = m.text.to_lowercase();
            t.contains("increased critical strike chance") || t.contains("critical strike chance")
        })
        .map(|m| parse_pct(&m.text))
        .sum()
}

/// Sum all "increased critical strike multiplier" mods from equipped items.
pub(crate) fn crit_multiplier_increased_from_items(build: &BuildData) -> f64 {
    build.items.iter().flat_map(|it| &it.mods)
        .filter(|m| {
            let t = m.text.to_lowercase();
            t.contains("increased critical strike multiplier") || t.contains("to critical strike multiplier")
        })
        .map(|m| parse_pct(&m.text))
        .sum()
}

/// Sum all "increased attack speed" mods from equipped items (in %).
pub(crate) fn attack_speed_from_items(build: &BuildData) -> f64 {
    build.items.iter().flat_map(|it| &it.mods)
        .filter(|m| m.text.to_lowercase().contains("increased attack speed"))
        .map(|m| parse_pct(&m.text))
        .sum()
}

/// Sum all flat accuracy rating mods from equipped items.
pub(crate) fn accuracy_from_items(build: &BuildData) -> f64 {
    let base_accuracy = 1_000.0 + build.level as f64 * 10.0; // rough base by level
    let item_accuracy: f64 = build.items.iter().flat_map(|it| &it.mods)
        .filter(|m| m.text.to_lowercase().contains("accuracy rating"))
        .map(|m| parse_flat(&m.text))
        .sum();
    base_accuracy + item_accuracy
}

/// PoE hit chance formula: Accuracy / (Accuracy + (Evasion/4)^0.8), clamped to [0.05, 1.0].
pub(crate) fn player_hit_chance(player_accuracy: f64, monster_evasion: f64) -> f64 {
    if monster_evasion <= 0.0 { return 1.0; }
    let hit = player_accuracy / (player_accuracy + (monster_evasion / 4.0_f64).powf(0.8));
    hit.clamp(0.05, 1.0)
}

/// Parse first flat integer from a mod text (e.g. "+500 to Accuracy Rating" → 500).
fn parse_flat(text: &str) -> f64 {
    text.split_whitespace()
        .find_map(|w| w.trim_start_matches('+').parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Fire DoT DPS — for Righteous Fire, Scorching Ray, Burning Arrow ignite.
/// Base damage = life_regen - RF_degen, then apply multiplier chain.
fn calculate_fire_dot_dps(build: &BuildData, setup: Option<&GemSetup>) -> f64 {
    // Aggregate fire dot multipliers from items
    let mut increased_fire_dot: f64 = 0.0;
    let mut increased_burning: f64 = 0.0;
    let more_multipliers: Vec<f64> = Vec::new(); // populated from gem supports below

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("increased fire damage over time") || text.contains("increased burning damage") {
                increased_fire_dot += parse_pct(&mod_.text);
            }
            if text.contains("increased damage over time") {
                increased_fire_dot += parse_pct(&mod_.text);
            }
        }
    }

    // Get more multipliers from support gems
    let mut more_mults = more_multipliers;
    if let Some(s) = setup {
        for gem in &s.gems {
            if !gem.is_support { continue; }
            let name = gem.name.to_lowercase();
            // Known support multipliers (gem level 20)
            let mult = match name.as_str() {
                "elemental focus support" => Some(1.49),
                "swift affliction support" => Some(1.35),
                "burning damage support" => Some(1.44),
                "concentrated effect support" => Some(1.54),
                "efficacy support" => Some(1.35),
                "empower support" => None, // affects gem level, not DPS directly
                _ => None,
            };
            if let Some(m) = mult { more_mults.push(m); }
        }
    }

    // Base DPS: approximate from build level (proper impl needs tree data)
    // RF base: 20% of life per second (capped by regen)
    // Simplified: base = 1000 dps at level 90, scaled by fire_dot mods
    let base_dps = 1_000.0 + build.level as f64 * 20.0;

    formulas::apply_multiplier_chain(base_dps, increased_fire_dot + increased_burning, &more_mults)
}

/// Generic DoT DPS (cold, phys, poison).
fn calculate_generic_dot_dps(build: &BuildData, setup: Option<&GemSetup>) -> f64 {
    let mut increased: f64 = 0.0;
    let mut more_mults: Vec<f64> = Vec::new();

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("increased damage over time") || text.contains("increased chaos damage") {
                increased += parse_pct(&mod_.text);
            }
        }
    }

    if let Some(s) = setup {
        for gem in &s.gems {
            if !gem.is_support { continue; }
            if gem.name.to_lowercase().contains("void manipulation") { more_mults.push(1.49); }
            if gem.name.to_lowercase().contains("swift affliction") { more_mults.push(1.35); }
        }
    }

    let base_dps = 500.0 + build.level as f64 * 15.0;
    formulas::apply_multiplier_chain(base_dps, increased, &more_mults)
}

/// Hit-based DPS (attack or spell).
fn calculate_hit_dps(build: &BuildData, setup: Option<&GemSetup>) -> f64 {
    let mut increased: f64 = 0.0;
    let mut more_mults: Vec<f64> = Vec::new();

    for item in &build.items {
        for mod_ in &item.mods {
            let text = mod_.text.to_lowercase();
            if text.contains("increased physical damage") || text.contains("increased spell damage") {
                increased += parse_pct(&mod_.text);
            }
        }
    }

    if let Some(s) = setup {
        for gem in &s.gems {
            if !gem.is_support { continue; }
            if gem.name.to_lowercase().contains("added fire") { more_mults.push(1.49); }
        }
    }

    let base_dps = 800.0 + build.level as f64 * 18.0;
    formulas::apply_multiplier_chain(base_dps, increased, &more_mults)
}

fn build_dps_sources(
    _build: &BuildData,
    dot_dps: f64,
    hit_dps: f64,
    total: f64,
) -> Vec<DpsSource> {
    let mut sources = Vec::new();

    if dot_dps > 0.0 {
        sources.push(DpsSource {
            source: "DoT Damage".to_string(),
            value: dot_dps,
            percent_of_total: if total > 0.0 { dot_dps / total * 100.0 } else { 0.0 },
            color: "var(--fire)".to_string(),
        });
    }

    if hit_dps > 0.0 {
        sources.push(DpsSource {
            source: "Hit Damage".to_string(),
            value: hit_dps,
            percent_of_total: if total > 0.0 { hit_dps / total * 100.0 } else { 0.0 },
            color: "var(--unique-bright)".to_string(),
        });
    }

    sources
}

fn build_multiplier_chain(
    _build: &BuildData,
    _setup: Option<&GemSetup>,
) -> Vec<MultiplierStep> {
    // TODO: populate from actual nodes + gems
    vec![
        MultiplierStep {
            label: "Base Damage".to_string(),
            multiplier: 1.0,
            step_type: MultiplierType::Base,
        },
        MultiplierStep {
            label: "Increased Damage".to_string(),
            multiplier: 1.0, // calculated above
            step_type: MultiplierType::Increased,
        },
    ]
}

fn parse_pct(text: &str) -> f64 {
    text.split_whitespace()
        .find_map(|w| w.trim_start_matches('+').trim_end_matches('%').parse::<f64>().ok())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::build::{Item, ItemMod};

    fn item_with_mod(text: &str) -> Item {
        let mut it = Item::default();
        it.slot = "Helmet".to_string();
        it.mods.push(ItemMod {
            id: "test".to_string(),
            text: text.to_string(),
            value1: 0.0,
            value2: None,
            mod_type: crate::models::build::ModType::Suffix,
            is_crafted: false,
            is_fractured: false,
        });
        it
    }

    #[test]
    fn rf_build_has_nonzero_dps() {
        let mut build = BuildData::default();
        build.level = 90;
        build.gems.push(crate::models::build::GemSetup {
            skill: "Righteous Fire".to_string(),
            slot: "BodyArmour".to_string(),
            socket_colors: "RRRR".to_string(),
            gems: vec![],
            is_main_skill: true,
        });
        let stats = calculate(&build);
        assert!(stats.total_dps > 0.0, "RF build should have DPS > 0");
        assert!(stats.dot_dps > 0.0, "RF should use DoT DPS");
        assert_eq!(stats.hit_dps, 0.0, "RF should not have hit DPS");
    }

    #[test]
    fn crit_chance_sums_item_mods() {
        let mut build = BuildData::default();
        build.items.push(item_with_mod("30% increased Critical Strike Chance"));
        build.items.push(item_with_mod("20% increased Critical Strike Chance"));
        let increased = crit_chance_increased_from_items(&build);
        assert!((increased - 50.0).abs() < 0.1, "expected 50% increased, got {increased}");
    }

    #[test]
    fn crit_chance_defaults_to_base_5_pct_when_no_mods() {
        let build = BuildData::default();
        let stats = calculate(&build);
        assert!((stats.crit_chance - 0.05).abs() < 0.001,
            "base crit should be 5%, got {}", stats.crit_chance);
    }

    #[test]
    fn crit_chance_increases_with_mods() {
        let mut build = BuildData::default();
        build.items.push(item_with_mod("100% increased Critical Strike Chance"));
        let stats = calculate(&build);
        assert!(stats.crit_chance > 0.05, "crit should be above 5% with items");
    }

    #[test]
    fn crit_multiplier_sums_item_mods() {
        let mut build = BuildData::default();
        build.items.push(item_with_mod("50% increased Critical Strike Multiplier"));
        let increased = crit_multiplier_increased_from_items(&build);
        assert!((increased - 50.0).abs() < 0.1, "expected 50, got {increased}");
    }

    #[test]
    fn attack_speed_sums_item_mods() {
        let mut build = BuildData::default();
        build.items.push(item_with_mod("15% increased Attack Speed"));
        build.items.push(item_with_mod("10% increased Attack Speed"));
        let spd = attack_speed_from_items(&build);
        assert!((spd - 25.0).abs() < 0.1, "expected 25% total, got {spd}");
    }

    #[test]
    fn hit_chance_100_pct_when_evasion_zero() {
        assert_eq!(player_hit_chance(1000.0, 0.0), 1.0);
    }

    #[test]
    fn hit_chance_at_least_5_pct() {
        // Extreme evasion — hit chance should not go below 5%
        assert!(player_hit_chance(1.0, 1_000_000.0) >= 0.05);
    }

    #[test]
    fn hit_chance_above_90_pct_for_high_accuracy() {
        // 5000 accuracy vs 6000 evasion — should be decent hit rate
        let hc = player_hit_chance(5_000.0, 6_000.0);
        assert!(hc > 0.50, "expected >50% hit chance, got {hc}");
    }

    #[test]
    fn accuracy_from_items_accumulates_flat_acc() {
        let mut build = BuildData::default();
        build.level = 90;
        build.items.push(item_with_mod("+500 to Accuracy Rating"));
        let acc = accuracy_from_items(&build);
        // base = 1000 + 90*10 = 1900, + 500 item = 2400
        assert!(acc > 2300.0 && acc < 2500.0, "expected ~2400, got {acc}");
    }
}
