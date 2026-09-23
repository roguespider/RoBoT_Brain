/// Confidence data contract — Per Architecture Chapter 19.
///
/// Defines domains for confidence tracking and utility functions
/// for decay and contradiction detection.
use serde::{Deserialize, Serialize};

/// The domain of a confidence score.
///
/// Per Architecture §19: every numeric score must have `confidence: f32`
/// (0.0–1.0) and the domain indicates what kind of entity it belongs to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConfidenceDomain {
    /// Confidence in a stored fact.
    Fact,
    /// Confidence in a relationship between two entities.
    Relationship,
    /// Confidence in an experience record.
    Experience,
    /// Confidence in a skill's effectiveness.
    Skill,
    /// Confidence in a workflow's expected outcome.
    Workflow,
    /// Confidence in a tool's reliability.
    Tool,
    /// Confidence in a planning strategy.
    Strategy,
}

impl std::fmt::Display for ConfidenceDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfidenceDomain::Fact => write!(f, "fact"),
            ConfidenceDomain::Relationship => write!(f, "relationship"),
            ConfidenceDomain::Experience => write!(f, "experience"),
            ConfidenceDomain::Skill => write!(f, "skill"),
            ConfidenceDomain::Workflow => write!(f, "workflow"),
            ConfidenceDomain::Tool => write!(f, "tool"),
            ConfidenceDomain::Strategy => write!(f, "strategy"),
        }
    }
}

/// Decay confidence over time using exponential decay.
///
/// Per Architecture §19: confidence decay uses `current * 0.5_f32.powf(hours * decay_rate)`.
/// This models the idea that confidence halves every `1/decay_rate` hours.
pub fn decay_confidence(current: f32, hours_since_update: f64, decay_rate: f32) -> f32 {
    let clamped = current.clamp(0.0, 1.0);
    clamped * 0.5_f32.powf((hours_since_update as f32) * decay_rate)
}

/// Detect if two confidence scores contradict each other.
///
/// Per Architecture §19: contradiction is flagged when the absolute
/// difference between two scores exceeds a threshold.
pub fn detect_contradiction(a: f32, b: f32, threshold: f32) -> bool {
    (a - b).abs() > threshold
}

/// Actively reference confidence functions to prevent dead-code warnings.
pub fn reference_confidence_contract() {
    let domain_ref = ConfidenceDomain::Fact;
    let display_ref = format!("{}", ConfidenceDomain::Tool);
    tracing::debug!(
        domain_ref = ?domain_ref,
        display_ref = %display_ref,
        "Confidence domain variants referenced"
    );
    let decayed = decay_confidence(0.8, 24.0, 0.1);
    tracing::debug!("Confidence decay reference: {:.4}", decayed);
    let contradiction = detect_contradiction(0.9, 0.3, 0.5);
    tracing::debug!("Contradiction reference: {}", contradiction);
}
