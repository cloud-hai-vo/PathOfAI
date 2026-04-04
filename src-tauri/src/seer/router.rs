use anyhow::Result;
use crate::models::build::BuildData;
use crate::models::analysis::AnalysisResult;
use crate::models::seer::SeerResponse;
use super::intent_classifier::classify;
use super::response_generator::{generate, engine_for_intent};

pub async fn route(
    question: &str,
    build: &BuildData,
    analysis: &AnalysisResult,
) -> Result<SeerResponse> {
    let intent = classify(question);
    let engine = engine_for_intent(&intent);
    let answer = generate(&intent, build, analysis);

    Ok(SeerResponse {
        answer,
        engine,
        confidence: 0.85,
        follow_up_questions: suggested_follow_ups(&intent),
        related_suggestions: vec![],
    })
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
