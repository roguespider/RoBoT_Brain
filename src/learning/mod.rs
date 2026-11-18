// src/learning/mod.rs

//! Learning module for experience-based learning
//!
//! Per Architecture §9 - Learning Pipeline:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection
//! Per Architecture §10 - Learning Engine pipeline

pub mod candidates;
pub mod confidence;
pub mod evolution;
pub mod extraction;
pub mod generalization;
pub mod hypothesis;
pub mod improvement;
pub mod lineage;
pub mod memory_state;
pub mod patterns;
pub mod pipeline;
pub mod promotion;
pub mod strategic;
pub mod types;
pub mod working_memory;

use crate::common::data_contracts::experience_record::ExperienceRecord;
use crate::learning::extraction::ExtractedKnowledge;
use crate::learning::improvement::SkillImprovement;
use crate::learning::patterns::Pattern;

/// Discover learning patterns from experiences by grouping by context signature.
///
/// Per Architecture §10.2 — Pattern recognition. Groups experiences by their
/// `context_signature` and returns a `Pattern` for each group that has at
/// least one entry.
pub fn pattern_discovery(experiences: &[ExperienceRecord]) -> Vec<Pattern> {
    crate::learning::patterns::detect_patterns(experiences, 1)
}

/// Extract knowledge rules from discovered patterns.
///
/// Per Architecture §10.3 — Knowledge extraction. Transforms each `Pattern`
/// into an `ExtractedKnowledge` record with a human-readable rule string.
pub fn extract_knowledge(patterns: &[Pattern]) -> Vec<ExtractedKnowledge> {
    crate::learning::extraction::extract_knowledge(patterns)
}

/// Compute a skill improvement given a skill ID and delta.
///
/// Per Architecture §10.5 — Skill improvement. Returns a `SkillImprovement`
/// record representing the metric change.
pub fn skill_improvement(skill_id: &str, delta: f32) -> SkillImprovement {
    let old_value = 0.5;
    let new_value = old_value + delta;
    SkillImprovement {
        skill_id: skill_id.to_string(),
        metric: "overall".to_string(),
        old_value,
        new_value,
        delta,
    }
}
