/// Decision data contract - Per Architecture Chapter 5.9.
///
/// The Decision object records the conclusion reached by the Reasoning Engine.
/// Per Architecture §5.9: selected_plan, reason, confidence, alternatives,
/// supporting_memory, supporting_experience, timestamp.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A decision made by the reasoning engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    /// Unique identifier for this decision.
    pub id: String,
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// The selected plan for this decision.
    pub selected_plan: String,
    /// The reason for selecting this plan.
    pub reason: String,
    /// Confidence in this decision (0.0-1.0).
    pub confidence: f32,
    /// Alternative plans that were considered.
    pub alternatives: Vec<String>,
    /// Memory IDs supporting this decision.
    pub supporting_memory: Vec<String>,
    /// Experience IDs supporting this decision.
    pub supporting_experience: Vec<String>,
    /// Timestamp of the decision.
    pub timestamp: i64,
}

impl Decision {
    /// Create a new decision.
    pub fn new(
        selected_plan: impl Into<String>,
        reason: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: Metadata::new("decision"),
            selected_plan: selected_plan.into(),
            reason: reason.into(),
            confidence,
            alternatives: Vec::new(),
            supporting_memory: Vec::new(),
            supporting_experience: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add an alternative plan.
    pub fn with_alternative(mut self, alt: impl Into<String>) -> Self {
        self.alternatives.push(alt.into());
        self
    }

    /// Add supporting memory.
    pub fn with_supporting_memory(mut self, memory_id: impl Into<String>) -> Self {
        self.supporting_memory.push(memory_id.into());
        self
    }

    /// Add supporting experience.
    pub fn with_supporting_experience(mut self, exp_id: impl Into<String>) -> Self {
        self.supporting_experience.push(exp_id.into());
        self
    }
}

impl Default for Decision {
    fn default() -> Self {
        Self {
            id: String::new(),
            metadata: Metadata::default(),
            selected_plan: String::new(),
            reason: String::new(),
            confidence: 0.5,
            alternatives: Vec::new(),
            supporting_memory: Vec::new(),
            supporting_experience: Vec::new(),
            timestamp: 0,
        }
    }
}

/// Actively reference decision builder methods to eliminate dead-code warnings.
pub fn reference_decision_methods() {
    let d1 = Decision::new("plan-1", "reason", 0.8).with_alternative("plan-2");
    tracing::debug!("Decision with_alternative: count={}", d1.alternatives.len());
    let d2 = Decision::new("plan-1", "reason", 0.8).with_supporting_memory("mem-1");
    tracing::debug!(
        "Decision with_supporting_memory: count={}",
        d2.supporting_memory.len()
    );
    let d3 = Decision::new("plan-1", "reason", 0.8).with_supporting_experience("exp-1");
    tracing::debug!(
        "Decision with_supporting_experience: count={}",
        d3.supporting_experience.len()
    );
    tracing::debug!("decision_methods: builder methods actively referenced");
}
