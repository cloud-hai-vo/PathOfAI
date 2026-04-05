use anyhow::Result;
use crate::core::cloud_ai::ApiKeyStore;
use crate::models::build::BuildData;
use crate::models::analysis::AnalysisResult;
use crate::models::seer::{SeerEngine, SeerResponse};
use super::intent_classifier::classify;
use super::response_generator::{generate, engine_for_intent};

/// Route a Seer question to the best available engine.
///
/// Priority:
///   1. Calculator / Knowledge templates (85% of queries — fast, offline)
///   2. Cloud AI fallback (when key configured and question is complex)
pub async fn route(
    question: &str,
    build: &BuildData,
    analysis: &AnalysisResult,
    ai_keys: Option<&ApiKeyStore>,
    http: Option<&reqwest::Client>,
) -> Result<SeerResponse> {
    let intent = classify(question);

    // Most queries are answered by our Calculator engine
    if !matches!(intent, super::intent_classifier::Intent::Fallback) {
        let engine = engine_for_intent(&intent);
        let answer = generate(&intent, build, analysis);
        return Ok(SeerResponse {
            answer,
            engine,
            confidence: 0.85,
            follow_up_questions: suggested_follow_ups(&intent),
            related_suggestions: vec![],
        });
    }

    // Fallback: try cloud AI if a provider is configured
    if let (Some(keys), Some(client)) = (ai_keys, http) {
        if keys.has_any_key() {
            // Build a context-rich prompt for the AI
            let prompt = build_seer_prompt(question, build, analysis);
            match crate::core::cloud_ai::cloud_query(&prompt, keys, client).await {
                Ok(answer) => {
                    return Ok(SeerResponse {
                        answer,
                        engine: SeerEngine::Cloud,
                        confidence: 0.75,
                        follow_up_questions: vec![],
                        related_suggestions: vec![],
                    });
                }
                Err(e) => {
                    log::warn!("Cloud AI fallback failed: {e}");
                }
            }
        }
    }

    // Last resort: static fallback answer
    Ok(SeerResponse {
        answer: fallback_answer(question, analysis),
        engine: SeerEngine::Fallback,
        confidence: 0.4,
        follow_up_questions: vec![
            "What is my DPS?".to_string(),
            "Are my resistances capped?".to_string(),
            "What should I upgrade first?".to_string(),
        ],
        related_suggestions: vec![],
    })
}

/// Build a concise prompt that gives the AI enough context without the full build JSON.
fn build_seer_prompt(question: &str, build: &BuildData, analysis: &AnalysisResult) -> String {
    format!(
        "Path of Exile build analysis context:\n\
         Build: {} (Lv{} {} {})\n\
         DPS: {} | Life: {} | ES: {} | Armour: {}\n\
         Resists: Fire {}% Cold {}% Lightning {}% Chaos {}%\n\
         Score: {}/100\n\
         Top issues: {}\n\n\
         Player question: {}\n\n\
         Give concise, actionable PoE advice.",
        analysis.build_name,
        build.level,
        build.class_name,
        build.ascendancy,
        analysis.offense.dps_label,
        analysis.defenses.life,
        analysis.defenses.energy_shield,
        analysis.defenses.armour,
        analysis.defenses.resistances.fire,
        analysis.defenses.resistances.cold,
        analysis.defenses.resistances.lightning,
        analysis.defenses.resistances.chaos,
        analysis.overall_score,
        analysis.issues.iter().take(2)
            .map(|i| i.title.as_str())
            .collect::<Vec<_>>()
            .join("; "),
        question,
    )
}

fn fallback_answer(question: &str, analysis: &AnalysisResult) -> String {
    format!(
        "I'm not sure how to answer \"{question}\" specifically, but here's what I know about your build:\n\n\
         • DPS: {}\n\
         • Life: {}\n\
         • Overall score: {}/100\n\n\
         Try asking about DPS, resistances, upgrades, or boss readiness.",
        analysis.offense.dps_label,
        analysis.defenses.life,
        analysis.overall_score,
    )
}

fn suggested_follow_ups(intent: &super::intent_classifier::Intent) -> Vec<String> {
    use super::intent_classifier::Intent::*;
    match intent {
        DpsQuery => vec![
            "What is my best DPS upgrade?".to_string(),
            "How do I improve my gem links?".to_string(),
        ],
        ResistQuery => vec![
            "What gear should I buy to cap resists?".to_string(),
            "Am I ready for elemental weakness maps?".to_string(),
        ],
        UpgradeQuery => vec![
            "What can I craft on my gear?".to_string(),
            "Am I ready for red maps?".to_string(),
        ],
        _ => vec![],
    }
}
