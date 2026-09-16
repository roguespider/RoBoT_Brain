/// LearningUpdate data contract - Per Architecture Chapter 10.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A learning update that modifies confidence in a target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningUpdate {
    /// Shared metadata (version, source, timestamp, correlation, confidence).
    pub metadata: Metadata,
    /// What is being updated (knowledge, skill, workflow, etc.).
    pub target_kind: String,
    /// The ID of the target being updated.
    pub target_id: String,
    /// Previous confidence score.
    pub old_confidence: f32,
    /// New confidence score after update.
    pub new_confidence: f32,
    /// Reason for the confidence change.
    pub reason: String,
}

impl LearningUpdate {
    pub fn new(
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
        old_confidence: f32,
        new_confidence: f32,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            metadata: Metadata::new("learning_update"),
            target_kind: target_kind.into(),
            target_id: target_id.into(),
            old_confidence,
            new_confidence,
            reason: reason.into(),
        }
    }
}
