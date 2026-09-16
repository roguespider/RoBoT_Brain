/// Decision data contract - Per Architecture Chapter 19.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A decision made by the reasoning engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Shared metadata (version, source, timestamp, correlation, confidence).
    pub metadata: Metadata,
    /// The action chosen.
    pub chosen_action: String,
    /// Alternative actions that were considered.
    pub alternatives: Vec<String>,
    /// Confidence in this decision (0.0-1.0).
    pub confidence: f32,
    /// Rationale for the decision.
    pub rationale: String,
}

impl Decision {
    pub fn new(action: impl Into<String>, rationale: impl Into<String>, confidence: f32) -> Self {
        Self {
            metadata: Metadata::new("decision"),
            chosen_action: action.into(),
            alternatives: Vec::new(),
            confidence,
            rationale: rationale.into(),
        }
    }
}
