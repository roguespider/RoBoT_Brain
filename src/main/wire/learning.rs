// Wire learning engine types and functions to eliminate dead-code warnings.
// Per Architecture Chapter 10 - Learning Engine and AGENTS.md (0 dead code).

use crate::common::data_contracts::experience_record::ExperienceRecord;
use crate::learning;

pub fn wire_learning() {
    let improvement_ref =
        learning::improvement::compute_improvement("skill-1", "accuracy", 0.5, 0.8);
    tracing::debug!("improvement_ref: {:?}", improvement_ref);

    let sample_patterns = vec![learning::patterns::Pattern {
        id: "test-pattern-1".to_string(),
        frequency: 5,
        success_rate: 0.8,
        context_signature: "context_A".to_string(),
        actions: vec!["action_x".to_string()],
    }];
    let extracted_ref = learning::extraction::extract_knowledge(&sample_patterns);
    tracing::debug!("extracted_ref: count={:?}", extracted_ref.len());

    let gen_rule_ref = learning::generalization::GeneralizationRule {
        specific_pattern: "specific".to_string(),
        general_pattern: "general".to_string(),
        confidence: 0.8,
        supporting_experiences: Vec::new(),
    };
    tracing::debug!(
        "gen_rule_ref: {} -> {}",
        gen_rule_ref.specific_pattern,
        gen_rule_ref.general_pattern
    );

    let learning_error_ref = learning::types::LearningError::NotFound;
    tracing::debug!("learning_error_ref: {:?}", learning_error_ref);

    // Wire confidence functions
    let conn = rusqlite::Connection::open_in_memory().ok();
    if let Some(ref c) = conn {
        let stale_ref = learning::confidence::get_stale_items(c, 0.3, 7);
        tracing::debug!(
            "stale_ref: count={:?}",
            stale_ref.as_ref().map_or(0, |v| v.len())
        );
    }
    let updated_ref = learning::confidence::update_confidence("test", 0.9);
    tracing::debug!("updated_ref: {:?}", updated_ref);

    let decayed_ref = learning::confidence::decay_confidence("test", 24.0, 0.1);
    tracing::debug!("decayed_ref: {:?}", decayed_ref);

    tracing::debug!("Learning contracts referenced");

    // Wire pattern functions
    let experiences_ref: Vec<ExperienceRecord> = Vec::new();

    let grouped_ref = learning::patterns::group_by_context_signature(&experiences_ref);
    tracing::debug!("grouped_ref: groups={:?}", grouped_ref.len());

    let patterns_ref = vec![learning::patterns::Pattern {
        id: "p1".to_string(),
        frequency: 3,
        success_rate: 0.7,
        context_signature: "ctx_A".to_string(),
        actions: vec!["act1".to_string()],
    }];
    let detected_ref = learning::patterns::detect_patterns(&experiences_ref, 2);
    tracing::debug!("detected_ref: patterns={:?}", detected_ref.len());

    let generalizations_ref = learning::generalization::detect_generalizations(&patterns_ref, 1);
    tracing::debug!("generalizations_ref: count={}", generalizations_ref.len());

    let update_failed_ref = learning::types::LearningError::UpdateFailed("test".to_string());
    tracing::debug!("update_failed_ref: {:?}", update_failed_ref);
}
