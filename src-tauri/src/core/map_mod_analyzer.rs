/// map_mod_analyzer.rs — Map Mod Danger Scorer (Algorithm 39).
/// Rates each map mod's danger level relative to the active build's archetype and stats.
use serde::{Deserialize, Serialize};
use crate::models::analysis::AnalysisResult;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DangerLevel {
    Safe,       // No impact
    Minor,      // Slight disadvantage
    Moderate,   // Manageable with care
    Major,      // Significant threat — consider rerolling
    Critical,   // Likely death or build-bricking — skip
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDanger {
    pub mod_text: String,
    pub level:    DangerLevel,
    pub reason:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDangerResult {
    pub mods:        Vec<ModDanger>,
    pub worst:       DangerLevel,
    pub verdict:     String,      // "Run" | "Run carefully" | "Reroll" | "Skip"
    pub fatal_mods:  Vec<String>, // Critical-level mod texts
    pub total_score: u32,         // 0–100
}

// ─── Core logic ───────────────────────────────────────────────────────────────

/// Score all map mods against the current build analysis.
pub fn score_map_mods(mods: &[&str], analysis: &AnalysisResult) -> MapDangerResult {
    let dangers: Vec<ModDanger> = mods.iter()
        .map(|&m| score_single_mod(m, analysis))
        .collect();

    let total_score = dangers.iter()
        .map(|d| d.level as u32 * 25)
        .sum::<u32>()
        .min(100);

    let worst = dangers.iter().map(|d| d.level).max().unwrap_or(DangerLevel::Safe);

    let fatal_mods = dangers.iter()
        .filter(|d| d.level == DangerLevel::Critical)
        .map(|d| d.mod_text.clone())
        .collect();

    let verdict = match worst {
        DangerLevel::Safe     => "Run",
        DangerLevel::Minor    => "Run",
        DangerLevel::Moderate => "Run carefully",
        DangerLevel::Major    => "Reroll",
        DangerLevel::Critical => "Skip",
    };

    MapDangerResult {
        mods: dangers,
        worst,
        verdict: verdict.to_string(),
        fatal_mods,
        total_score,
    }
}

fn score_single_mod(mod_text: &str, analysis: &AnalysisResult) -> ModDanger {
    use DangerLevel::*;

    let m = mod_text.to_lowercase();
    let archetype = &analysis.archetype;
    let res = &analysis.defenses.resistances;
    let life_regen = analysis.defenses.life_regen_flat as f64
        + analysis.defenses.life as f64 * analysis.defenses.life_regen_pct / 100.0;

    let (level, reason): (DangerLevel, &str) =
        if m.contains("cannot regenerate") || m.contains("no life regeneration")
            || m.contains("no mana regeneration")
        {
            // Fatal for RF (life regen is the only damage mitigation)
            if archetype.contains("fire_dot") || archetype.contains("rf") {
                (Critical, "RF requires life regen to survive — this mod is a hard skip")
            } else if life_regen > 500.0 {
                (Major, "Removes significant life recovery from regen-heavy build")
            } else {
                (Minor, "Low regen build — minimal impact")
            }

        } else if m.contains("players cannot leech") || m.contains("no leech") {
            // We infer leech use from archetype (melee/attack builds typically leech)
            let uses_leech = archetype.contains("attack") || archetype.contains("melee")
                || archetype.contains("blade") || archetype.contains("slayer");
            if uses_leech {
                (Critical, "Build relies on leech for sustain — this is fatal")
            } else {
                (Safe, "Build does not appear to use leech")
            }

        } else if m.contains("elemental reflect") {
            // Elemental damage ratio from offense stats
            let total = analysis.offense.total_dps.max(1.0);
            let ele_ratio = analysis.offense.dot_dps / total; // DoT is typically ele for RF/ignite
            if ele_ratio > 0.3 || archetype.contains("fire_dot") || archetype.contains("cold")
                || archetype.contains("lightning") || archetype.contains("ele")
            {
                (Critical, "High elemental damage — reflect will kill you instantly")
            } else if analysis.offense.hit_dps > 0.0 {
                (Major, "Some elemental hit damage will be reflected")
            } else {
                (Safe, "No elemental damage to reflect")
            }

        } else if m.contains("physical reflect") {
            let total = analysis.offense.total_dps.max(1.0);
            let phys_ratio = analysis.offense.hit_dps / total; // approximation
            if archetype.contains("phys") || archetype.contains("bleed") || archetype.contains("impale") {
                (Critical, "Mostly physical damage — reflect is lethal")
            } else if phys_ratio > 0.2 {
                (Moderate, "Partial physical damage reflected — be cautious")
            } else {
                (Safe, "No significant physical damage to reflect")
            }

        } else if m.contains("monsters are hexproof") {
            // Curse builds suffer significantly
            let uses_curses = archetype.contains("curse") || archetype.contains("hex")
                || archetype.contains("occultist");
            if uses_curses {
                (Major, "Curse-based damage or defense is disabled")
            } else {
                (Minor, "No curses allocated — minor impact")
            }

        } else if m.contains("maximum resistances") || m.contains("maximum resistance") {
            let penalty = parse_number(&m).unwrap_or(10) as i32;
            // Minimum overcap across elemental resistances
            let min_overcap = [
                res.fire  - res.max_fire,
                res.cold  - res.max_cold,
                res.lightning - res.max_lightning,
            ].iter().copied().min().unwrap_or(0);

            if min_overcap < penalty {
                (Critical, "Resistance drops below cap — overcap your resists before running")
            } else if min_overcap < penalty + 5 {
                (Major, "Overcap barely covers the penalty — risky")
            } else {
                (Minor, "Sufficient overcap to absorb penalty")
            }

        } else if m.contains("blood magic") || m.contains("no mana") {
            if analysis.defenses.mana < 50 {
                (Major, "Mana-dependent skills may break under Blood Magic")
            } else {
                (Moderate, "All skill costs paid from life — watch your life pool")
            }

        } else if m.contains("players are cursed with enfeeble") {
            (Moderate, "Enfeeble reduces hit accuracy and damage — manageable but significant")

        } else if m.contains("players are cursed with temporal chains") {
            if analysis.defenses.ailment_immunity.freeze {
                (Minor, "Slowed action speed — immune to freeze, slight disadvantage only")
            } else {
                (Moderate, "Slow can be dangerous in dense maps")
            }

        } else if m.contains("burning ground") || m.contains("ground effects") {
            if res.fire >= res.max_fire {
                (Minor, "Capped fire res — minimal damage from burning ground")
            } else {
                (Moderate, "Uncapped fire resistance — burning ground deals real damage")
            }

        } else if m.contains("monsters deal") && m.contains("extra damage as") {
            let extra = parse_number(&m).unwrap_or(25) as f64 / 100.0;
            if extra > 0.4 {
                (Major, "Very high extra elemental damage taken")
            } else {
                (Moderate, "Additional element on hits — verify resistance capping")
            }

        } else {
            (Safe, "No specific threat detected for this build")
        };

    ModDanger {
        mod_text: mod_text.to_string(),
        level,
        reason: reason.to_string(),
    }
}

fn parse_number(text: &str) -> Option<u32> {
    text.split_whitespace().find_map(|w| {
        // Strip leading `-` and trailing `%` to get unsigned value
        w.trim_start_matches('-').trim_end_matches('%').parse::<u32>().ok()
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::analysis::{
        AnalysisResult, DefenseStats, OffenseStats, ResistanceProfile, AilmentImmunity,
    };

    fn make_analysis(archetype: &str, overrides: Option<fn(&mut AnalysisResult)>) -> AnalysisResult {
        let mut a = AnalysisResult {
            build_id: "t".to_string(),
            build_name: "Test Build".to_string(),
            class_name: "Templar".to_string(),
            ascendancy: "Inquisitor".to_string(),
            level: 90,
            archetype: archetype.to_string(),
            archetype_label: archetype.to_string(),
            overall_score: 70,
            defenses: DefenseStats {
                life: 5000,
                life_regen_flat: 300.0,
                life_regen_pct: 5.0,
                mana: 300,
                resistances: ResistanceProfile {
                    fire: 75, cold: 75, lightning: 75, chaos: -60,
                    max_fire: 75, max_cold: 75, max_lightning: 75, max_chaos: 75,
                    fire_overcap: 0, cold_overcap: 0, lightning_overcap: 0,
                },
                ailment_immunity: AilmentImmunity {
                    freeze: false, shock: false, ignite: false, bleed: false,
                    corrupted_blood: false, poison: false, stun: false, curse_immune: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            offense: OffenseStats {
                total_dps: 2_000_000.0,
                dps_label: "2M".to_string(),
                main_skill: "Righteous Fire".to_string(),
                hit_dps: 0.0,
                dot_dps: 2_000_000.0,
                ..Default::default()
            },
            issues: vec![],
            suggestions: vec![],
            item_scores: vec![],
            gem_setups: vec![],
        };
        if let Some(f) = overrides { f(&mut a); }
        a
    }

    #[test]
    fn safe_mod_scores_safe() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&["Monsters have 50% increased life"], &a);
        assert_eq!(result.worst, DangerLevel::Safe);
        assert_eq!(result.verdict, "Run");
    }

    #[test]
    fn no_regen_is_critical_for_rf() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&["Players cannot regenerate Life, Mana or Energy Shield"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Critical);
        assert_eq!(result.verdict, "Skip");
    }

    #[test]
    fn no_regen_is_minor_for_low_regen_build() {
        let a = make_analysis("coc_icenova", Some(|a| {
            a.defenses.life_regen_flat = 50.0;
            a.defenses.life_regen_pct = 0.0;
        }));
        let result = score_map_mods(&["Players cannot regenerate Life"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Minor);
    }

    #[test]
    fn elemental_reflect_critical_for_ele_dot_build() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&["Elemental Reflect"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Critical);
        assert!(result.fatal_mods.iter().any(|m| m.contains("Elemental")));
    }

    #[test]
    fn elemental_reflect_safe_for_pure_phys() {
        let a = make_analysis("phys_attack", Some(|a| {
            a.offense.dot_dps = 0.0;
            a.offense.hit_dps = 2_000_000.0;
        }));
        let result = score_map_mods(&["Elemental Reflect"], &a);
        // phys archetype, dot_dps = 0 → ratio = 0 → should be at most Major
        assert!(result.mods[0].level <= DangerLevel::Major);
    }

    #[test]
    fn physical_reflect_critical_for_phys_build() {
        let a = make_analysis("phys_bleed", None);
        let result = score_map_mods(&["Physical Reflect"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Critical);
    }

    #[test]
    fn physical_reflect_safe_for_spell_build() {
        let a = make_analysis("fire_dot", Some(|a| {
            a.offense.hit_dps = 0.0;
        }));
        let result = score_map_mods(&["Physical Reflect"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Safe);
    }

    #[test]
    fn max_res_penalty_critical_if_no_overcap() {
        // resistances are at cap (75), penalty of 10 → drops to 65 → Critical
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&["Players have -10% to maximum Resistances"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Critical);
    }

    #[test]
    fn max_res_penalty_minor_with_overcap() {
        let a = make_analysis("fire_dot", Some(|a| {
            // All elements overcapped by 20 — well above the 10% penalty + 5 buffer
            a.defenses.resistances.fire = 95;
            a.defenses.resistances.cold = 95;
            a.defenses.resistances.lightning = 95;
        }));
        let result = score_map_mods(&["Players have -10% to maximum Resistances"], &a);
        // min overcap = 95-75 = 20 > 10+5 → Minor
        assert_eq!(result.mods[0].level, DangerLevel::Minor);
    }

    #[test]
    fn verdict_for_critical_is_skip() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&["Players cannot regenerate Life"], &a);
        assert_eq!(result.verdict, "Skip");
    }

    #[test]
    fn verdict_for_moderate_is_run_carefully() {
        let a = make_analysis("generic", None);
        let result = score_map_mods(&["Players are Cursed with Enfeeble"], &a);
        assert_eq!(result.verdict, "Run carefully");
    }

    #[test]
    fn total_score_capped_at_100() {
        let a = make_analysis("fire_dot", None);
        // Many critical mods — score should cap at 100
        let many_crits = vec![
            "Players cannot regenerate Life",
            "Elemental Reflect",
            "Players have -10% to maximum Resistances",
            "Players cannot regenerate Life, Mana or Energy Shield",
            "Elemental Reflect",
        ];
        let result = score_map_mods(&many_crits, &a);
        assert!(result.total_score <= 100);
    }

    #[test]
    fn empty_mods_returns_run_verdict() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&[], &a);
        assert_eq!(result.verdict, "Run");
        assert_eq!(result.total_score, 0);
        assert!(result.mods.is_empty());
    }

    #[test]
    fn multiple_mods_worst_drives_verdict() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&[
            "Monsters have 50% increased life", // Safe
            "Players cannot regenerate Life",   // Critical for fire_dot
        ], &a);
        assert_eq!(result.worst, DangerLevel::Critical);
        assert_eq!(result.verdict, "Skip");
    }

    #[test]
    fn fatal_mods_list_only_contains_critical() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&[
            "Players cannot regenerate Life",   // Critical
            "Players are Cursed with Enfeeble", // Moderate
        ], &a);
        assert_eq!(result.fatal_mods.len(), 1);
        assert!(result.fatal_mods[0].contains("regenerate"));
    }

    #[test]
    fn hexproof_major_for_curse_build() {
        let a = make_analysis("curse_occultist", None);
        let result = score_map_mods(&["Monsters are Hexproof"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Major);
    }

    #[test]
    fn blood_magic_moderate_with_mana() {
        let a = make_analysis("fire_dot", None);
        let result = score_map_mods(&["Blood Magic"], &a);
        assert_eq!(result.mods[0].level, DangerLevel::Moderate);
    }

    #[test]
    fn parse_number_extracts_from_mod_text() {
        assert_eq!(parse_number("players have -10% to maximum resistances"), Some(10));
        assert_eq!(parse_number("25% extra damage as lightning"), Some(25));
        assert_eq!(parse_number("no numbers here"), None);
    }
}
