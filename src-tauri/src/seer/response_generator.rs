use crate::models::analysis::AnalysisResult;
use crate::models::build::BuildData;
use crate::models::seer::SeerEngine;
use super::intent_classifier::Intent;

pub fn generate(intent: &Intent, build: &BuildData, analysis: &AnalysisResult) -> String {
    match intent {
        Intent::DpsQuery => format!(
            "Your total DPS is **{}**.\n\n\
             Main skill: {} ({})\n\
             Top upgrade for DPS: {}",
            analysis.offense.dps_label,
            analysis.offense.main_skill,
            analysis.offense.sources.first().map(|s| format!("{:.0}% {}", s.percent_of_total, s.source)).unwrap_or_default(),
            analysis.suggestions.first().map(|s| s.title.as_str()).unwrap_or("no suggestions available"),
        ),

        Intent::ResistQuery => {
            let res = &analysis.defenses.resistances;
            let uncapped: Vec<String> = [
                ("Fire", res.fire, res.max_fire),
                ("Cold", res.cold, res.max_cold),
                ("Lightning", res.lightning, res.max_lightning),
            ]
            .iter()
            .filter(|(_, val, max)| val < max)
            .map(|(name, val, max)| format!("{name}: {val}%/{max}%"))
            .collect();

            if uncapped.is_empty() {
                format!(
                    "Your elemental resistances are all capped. ✓\n\
                     Fire {fire}% | Cold {cold}% | Lightning {lightning}%\n\
                     Chaos: {chaos}%{chaos_warn}",
                    fire = res.fire, cold = res.cold, lightning = res.lightning,
                    chaos = res.chaos,
                    chaos_warn = if res.chaos < 0 { " ⚠️ negative" } else { "" }
                )
            } else {
                format!(
                    "You have uncapped resistances:\n{}\n\n\
                     Fix: add resistance to jewelry or craft it on boots/gloves.",
                    uncapped.join("\n")
                )
            }
        }

        Intent::LifeQuery => format!(
            "You have **{} life** ({}ES).\n\n\
             {}",
            analysis.defenses.life,
            analysis.defenses.energy_shield,
            if analysis.defenses.life < 4_500 {
                format!("⚠️ Below recommended 4,500 for mapping. Aim for 5,000+ for endgame.")
            } else {
                "✓ Life pool is solid for most content.".to_string()
            }
        ),

        Intent::UpgradeQuery => {
            let top = analysis.suggestions.iter().take(3).enumerate()
                .map(|(i, s)| format!("{}. {} — {}", i + 1, s.title, s.detail))
                .collect::<Vec<_>>()
                .join("\n");

            if top.is_empty() {
                "Your build looks solid — no major upgrades detected.".to_string()
            } else {
                format!("Top upgrades for your build:\n\n{top}")
            }
        }

        Intent::BossQuery => format!(
            "Based on your stats ({} DPS, {} life):\n\n\
             Check the Blood Pact panel for per-boss readiness.",
            analysis.offense.dps_label,
            analysis.defenses.life
        ),

        Intent::Fallback => {
            "That question needs a connected AI to answer well.\n\
             [Connect Claude →] in Settings to unlock creative Seer answers.\n\n\
             I can answer questions about your DPS, resistances, life, upgrades, gems, bosses, and map mods with our built-in calculator.".to_string()
        }

        _ => format!(
            "Calculating answer for your {} build…\n\
             Check the relevant panel for details.",
            analysis.archetype_label
        ),
    }
}

pub fn engine_for_intent(intent: &Intent) -> SeerEngine {
    match intent {
        Intent::DpsQuery | Intent::ResistQuery | Intent::LifeQuery => SeerEngine::Calculator,
        Intent::UpgradeQuery | Intent::GemQuery | Intent::MapModQuery => SeerEngine::Calculator,
        Intent::CraftQuery | Intent::BossQuery | Intent::PassiveTreeQuery => SeerEngine::Knowledge,
        Intent::Fallback => SeerEngine::Fallback,
        _ => SeerEngine::Knowledge,
    }
}
