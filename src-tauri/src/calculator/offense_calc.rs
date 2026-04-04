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

    OffenseStats {
        total_dps,
        dps_label: formulas::format_dps(total_dps),
        main_skill,
        hit_dps,
        dot_dps,
        crit_chance: 0.0,       // TODO: calculate from crit nodes + gear
        crit_multiplier: 1.5,   // default 150%
        attack_speed: 0.0,
        cast_speed: 0.0,
        hit_chance: 1.0,        // TODO: accuracy check
        sources,
        multiplier_chain,
    }
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
}
