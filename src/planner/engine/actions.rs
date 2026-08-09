// src/planner/engine/actions.rs
//! Action selection and scoring for the planning engine

use super::types::{
    ActionCandidate, KnowledgeRef, PlannerPolicy, RiskLevel,
};

/// Score an action candidate based on policy
pub fn score_action(action: &ActionCandidate, policy: &PlannerPolicy) -> f32 {
    let mut score = 0.0;

    // Factor in supporting knowledge confidence
    if !action.supporting_knowledge.is_empty() {
        let avg_confidence = calculate_knowledge_confidence(&action.supporting_knowledge);

        if avg_confidence >= policy.min_knowledge_confidence {
            score += policy.knowledge_weight * avg_confidence;
        }
    }

    // Factor in relevant past experiences
    if action.past_experiences.len() >= policy.min_experience_count as usize {
        let success_count: f32 = action
            .past_experiences
            .iter()
            .map(|e| if e.was_successful { 1.0 } else { 0.0 })
            .sum();
        let success_rate = success_count / action.past_experiences.len() as f32;
        score += policy.experience_weight * success_rate;
    }

    // Factor in direct confidence
    score += policy.confidence_weight * action.confidence;

    // Penalize higher-risk actions so risk_level participates in scoring.
    score -= match action.risk_level {
        RiskLevel::Low => 0.0,
        RiskLevel::Medium => 0.1,
        RiskLevel::High => 0.25,
        RiskLevel::Critical => 0.4,
    };

    score.clamp(0.0, 1.0)
}

/// Select the best action from a list of candidates based on scoring
pub fn select_best_scored(
    actions: Vec<(ActionCandidate, f32)>,
) -> Option<ActionCandidate> {
    if actions.is_empty() {
        return None;
    }

    let mut scored = actions;
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    scored.into_iter().next().map(|(action, _)| action)
}

/// Calculate knowledge confidence from a list of knowledge references
pub fn calculate_knowledge_confidence(knowledge: &[KnowledgeRef]) -> f32 {
    if knowledge.is_empty() {
        return 0.0;
    }
    let total: f32 = knowledge.iter().map(|k| k.confidence).sum();
    total / knowledge.len() as f32
}
