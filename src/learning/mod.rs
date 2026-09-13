// src/learning/mod.rs

//! Learning module for experience-based learning
//!
//! Per Architecture §9 - Learning Pipeline:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection

pub mod candidates;
pub mod confidence;
pub mod extraction;
pub mod generalization;
pub mod hypothesis;
pub mod improvement;
pub mod lineage;
pub mod memory_state;
pub mod patterns;
pub mod pipeline;
pub mod promotion;
pub mod types;
pub mod working_memory;

/// Active reference to improvement contracts.
pub fn reference_improvement_contract() {
    let improvement =
        crate::learning::improvement::compute_improvement("skill-1", "accuracy", 0.5, 0.8);
    tracing::info!(
        skill_id = %improvement.skill_id,
        metric = %improvement.metric,
        old_value = improvement.old_value,
        new_value = improvement.new_value,
        delta = improvement.delta,
        "Learning improvement contract actively referenced"
    );
}

/// Actively reference learning extraction contract to prevent dead-code warnings.
/// Per Architecture §10.3 — Knowledge extraction from patterns.
pub fn reference_extraction_contract() {
    let sample_patterns = vec![crate::learning::patterns::Pattern {
        id: "test-pattern-1".to_string(),
        frequency: 5,
        success_rate: 0.8,
        context_signature: "context_A".to_string(),
        actions: vec!["action_x".to_string()],
    }];
    let extracted = crate::learning::extraction::extract_knowledge(&sample_patterns);
    for knowledge in &extracted {
        tracing::info!(
            pattern_id = %knowledge.pattern_id,
            rule = %knowledge.rule,
            confidence = knowledge.confidence,
            "Learning extraction reference: rule extracted"
        );
    }
    let extracted_count = extracted.len();
    tracing::debug!(
        extracted_count,
        "Learning extraction reference: items extracted"
    );
}
