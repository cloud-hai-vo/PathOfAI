use crate::models::analysis::AnalysisResult;
use crate::models::build::BuildData;
use crate::models::seer::SeerEngine;
use super::intent_classifier::Intent;

pub fn generate(intent: &Intent, _build: &BuildData, analysis: &AnalysisResult) -> String {
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

        Intent::GemQuery => {
            let gem_count: usize = analysis.gem_setups.iter().map(|g| g.gems.len()).sum();
            let main = analysis.gem_setups.iter().find(|g| g.is_main_skill);
            let main_info = main.map(|s| {
                let supports: Vec<&str> = s.gems.iter()
                    .filter(|g| g.is_support)
                    .map(|g| g.name.as_str())
                    .collect();
                format!(
                    "Main link: **{}** with {} support(s): {}",
                    s.skill,
                    supports.len(),
                    if supports.is_empty() { "none".to_string() } else { supports.join(", ") }
                )
            }).unwrap_or_else(|| "No main skill detected.".to_string());

            format!(
                "{main_info}\n\n\
                 Total gems equipped: {gem_count}.\n\
                 Check the Gems panel for full details and level/quality suggestions."
            )
        }

        Intent::FlaskQuery => {
            let ai = &analysis.defenses.ailment_immunity;
            let mut covered = Vec::new();
            let mut missing = Vec::new();
            for (name, immune) in [("Freeze", ai.freeze), ("Shock", ai.shock), ("Ignite", ai.ignite), ("Bleed", ai.bleed)] {
                if immune { covered.push(name); } else { missing.push(name); }
            }
            let status = if missing.is_empty() {
                "✓ All ailments covered by your gear.".to_string()
            } else {
                format!("⚠️ Missing ailment coverage: {}. Use a flask or unique item.", missing.join(", "))
            };
            format!(
                "Flask setup for your {} build:\n\n\
                 {status}\n\n\
                 Recommended: Life flask + Quicksilver + ailment immunity flasks.\n\
                 For RF builds: Cinderswallow Urn is BiS for ignite immunity + DPS bonus.",
                analysis.archetype_label
            )
        }

        Intent::PriceQuery => {
            let top_suggestion = analysis.suggestions.first()
                .map(|s| format!(
                    "Top upgrade: **{}** (estimated ~{:.1} div)",
                    s.title, s.estimated_cost_div
                ))
                .unwrap_or_else(|| "No specific upgrade flagged.".to_string());
            format!(
                "{top_suggestion}\n\n\
                 Use the **Forge** panel for full upgrade costs and trade links.\n\
                 Prices update from poe.ninja every 5 minutes."
            )
        }

        Intent::CraftQuery => {
            format!(
                "For your {} build, the most cost-effective crafting approach:\n\n\
                 • **Bench craft**: deterministic — guarantees the mod (1-2c)\n\
                 • **Essence spam**: guarantees one mod, others random (~50-200c)\n\
                 • **Chaos spam**: full reroll — high variance, ~200+ attempts for good mods\n\n\
                 Check the **Forge** panel for slot-specific craft vs buy comparisons.",
                analysis.archetype_label
            )
        }

        Intent::PassiveTreeQuery => {
            let allocated = analysis.defenses.life; // proxy for build progress
            let tree_count = if allocated > 0 {
                format!("Your build has {} life — tree contributes significantly.", allocated)
            } else {
                "Import a build to see tree analysis.".to_string()
            };
            format!(
                "{tree_count}\n\n\
                 Check the **Passive Tree** panel for node recommendations.\n\
                 For {} builds, prioritize keystones that match your archetype.",
                analysis.archetype_label
            )
        }

        Intent::CompareQuery => {
            format!(
                "To compare two builds, use the **Codex** panel.\n\n\
                 It shows stat deltas (DPS, life, resistances) and tree overlap \
                 between any two imported builds."
            )
        }

        Intent::MapModQuery => {
            format!(
                "For your {} build, dangerous map mods:\n\n\
                 • **No Regeneration** — dangerous without leech\n\
                 • **Elemental Reflect** — skip for spell/DoT builds\n\
                 • **-% Max Resistances** — take extra care\n\n\
                 Check the **Curse & Map** panel for a full mod danger rating.",
                analysis.archetype_label
            )
        }

        Intent::Fallback => {
            "That question needs a connected AI to answer well.\n\
             [Connect Claude →] in Settings to unlock creative Seer answers.\n\n\
             I can answer questions about your DPS, resistances, life, upgrades, gems, bosses, and map mods with our built-in calculator.".to_string()
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::build::BuildData;
    use crate::models::analysis::{AnalysisResult, DefenseStats, OffenseStats, ResistanceProfile, EffectiveHP, AilmentImmunity};

    fn empty_analysis() -> AnalysisResult {
        AnalysisResult {
            build_id: "test".to_string(),
            build_name: "RF Inquisitor".to_string(),
            class_name: "Templar".to_string(),
            ascendancy: "Inquisitor".to_string(),
            level: 90,
            archetype: "fire_dot".to_string(),
            archetype_label: "RF Inquisitor".to_string(),
            overall_score: 72,
            defenses: DefenseStats {
                life: 5500,
                resistances: ResistanceProfile { fire: 75, cold: 75, lightning: 75, chaos: -60, max_fire: 75, max_cold: 75, max_lightning: 75, max_chaos: 75, fire_overcap: 0, cold_overcap: 0, lightning_overcap: 0 },
                ailment_immunity: AilmentImmunity { freeze: true, shock: false, ignite: true, bleed: false, corrupted_blood: false, poison: false, stun: false, curse_immune: false, ..Default::default() },
                ..Default::default()
            },
            offense: OffenseStats {
                total_dps: 2_500_000.0,
                dps_label: "2.50M".to_string(),
                main_skill: "Righteous Fire".to_string(),
                ..Default::default()
            },
            issues: vec![],
            suggestions: vec![],
            item_scores: vec![],
            gem_setups: vec![],
        }
    }

    fn build() -> BuildData { BuildData::default() }

    #[test]
    fn flask_query_mentions_missing_ailments() {
        let analysis = empty_analysis(); // shock + bleed not covered
        let resp = generate(&Intent::FlaskQuery, &build(), &analysis);
        assert!(resp.to_lowercase().contains("shock") || resp.to_lowercase().contains("bleed"),
            "Flask response should mention uncovered ailments");
    }

    #[test]
    fn flask_query_all_covered_says_covered() {
        let mut analysis = empty_analysis();
        analysis.defenses.ailment_immunity.shock = true;
        analysis.defenses.ailment_immunity.bleed = true;
        let resp = generate(&Intent::FlaskQuery, &build(), &analysis);
        assert!(resp.contains("covered") || resp.contains("✓"),
            "All ailments covered → response should say so");
    }

    #[test]
    fn price_query_mentions_forge_panel() {
        let analysis = empty_analysis();
        let resp = generate(&Intent::PriceQuery, &build(), &analysis);
        assert!(resp.to_lowercase().contains("forge") || resp.to_lowercase().contains("poe.ninja"),
            "Price query should mention Forge panel or poe.ninja");
    }

    #[test]
    fn craft_query_mentions_benchcraft() {
        let analysis = empty_analysis();
        let resp = generate(&Intent::CraftQuery, &build(), &analysis);
        assert!(resp.to_lowercase().contains("bench") || resp.to_lowercase().contains("craft"),
            "Craft query should mention bench craft");
    }

    #[test]
    fn passive_tree_query_mentions_passive_panel() {
        let analysis = empty_analysis();
        let resp = generate(&Intent::PassiveTreeQuery, &build(), &analysis);
        assert!(resp.to_lowercase().contains("passive") || resp.to_lowercase().contains("tree"),
            "Passive tree query should reference the tree panel");
    }

    #[test]
    fn compare_query_mentions_codex_panel() {
        let analysis = empty_analysis();
        let resp = generate(&Intent::CompareQuery, &build(), &analysis);
        assert!(resp.to_lowercase().contains("codex") || resp.to_lowercase().contains("compare"),
            "Compare query should mention Codex panel");
    }

    #[test]
    fn map_mod_query_mentions_curse_map_panel() {
        let analysis = empty_analysis();
        let resp = generate(&Intent::MapModQuery, &build(), &analysis);
        assert!(resp.to_lowercase().contains("map") || resp.to_lowercase().contains("mod"),
            "Map mod query should mention map mods");
    }

    #[test]
    fn fallback_mentions_connect_ai() {
        let analysis = empty_analysis();
        let resp = generate(&Intent::Fallback, &build(), &analysis);
        assert!(resp.to_lowercase().contains("connect") || resp.to_lowercase().contains("ai") || resp.to_lowercase().contains("settings"),
            "Fallback should mention connecting AI");
    }

    #[test]
    fn all_intents_return_non_empty_response() {
        let analysis = empty_analysis();
        let b = build();
        for intent in [
            Intent::DpsQuery, Intent::ResistQuery, Intent::LifeQuery,
            Intent::UpgradeQuery, Intent::GemQuery, Intent::FlaskQuery,
            Intent::PriceQuery, Intent::CraftQuery, Intent::PassiveTreeQuery,
            Intent::CompareQuery, Intent::MapModQuery, Intent::BossQuery,
            Intent::Fallback,
        ] {
            let resp = generate(&intent, &b, &analysis);
            assert!(!resp.is_empty(), "{intent:?} should return a non-empty response");
        }
    }

    #[test]
    fn engine_for_flask_is_calculator() {
        assert!(matches!(engine_for_intent(&Intent::FlaskQuery), SeerEngine::Calculator));
    }

    #[test]
    fn engine_for_craft_is_knowledge() {
        assert!(matches!(engine_for_intent(&Intent::CraftQuery), SeerEngine::Knowledge));
    }

    #[test]
    fn engine_for_fallback_is_fallback() {
        assert!(matches!(engine_for_intent(&Intent::Fallback), SeerEngine::Fallback));
    }
}

pub fn engine_for_intent(intent: &Intent) -> SeerEngine {
    match intent {
        Intent::DpsQuery | Intent::ResistQuery | Intent::LifeQuery => SeerEngine::Calculator,
        Intent::UpgradeQuery | Intent::GemQuery | Intent::MapModQuery => SeerEngine::Calculator,
        Intent::FlaskQuery | Intent::PriceQuery | Intent::CompareQuery => SeerEngine::Calculator,
        Intent::CraftQuery | Intent::BossQuery | Intent::PassiveTreeQuery => SeerEngine::Knowledge,
        Intent::Fallback => SeerEngine::Fallback,
    }
}
