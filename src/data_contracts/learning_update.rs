/// LearningUpdate data contract - Per Architecture Chapter 5.12 and Chapter 10.
///
/// LearningUpdate describes changes that should be applied after reflection.
/// Per Architecture §5.12: create memory, update confidence, strengthen relationship,
/// weaken relationship, create experience, refine workflow, archive obsolete knowledge.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// The kind of learning update action.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LearningAction {
    /// Create a new memory.
    CreateMemory,
    /// Update confidence in a target.
    UpdateConfidence,
    /// Strengthen a relationship.
    StrengthenRelationship,
    /// Weaken a relationship.
    WeakenRelationship,
    /// Create a new experience.
    CreateExperience,
    /// Refine an existing workflow.
    RefineWorkflow,
    /// Archive obsolete knowledge.
    ArchiveObsoleteKnowledge,
}

impl std::fmt::Display for LearningAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LearningAction::CreateMemory => "create_memory",
            LearningAction::UpdateConfidence => "update_confidence",
            LearningAction::StrengthenRelationship => "strengthen_relationship",
            LearningAction::WeakenRelationship => "weaken_relationship",
            LearningAction::CreateExperience => "create_experience",
            LearningAction::RefineWorkflow => "refine_workflow",
            LearningAction::ArchiveObsoleteKnowledge => "archive_obsolete_knowledge",
        };
        write!(f, "{}", s)
    }
}

/// A learning update that modifies a target after reflection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningUpdate {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// The action to perform.
    pub action: LearningAction,
    /// What is being updated (knowledge, skill, workflow, etc.).
    pub target_kind: String,
    /// The ID of the target being updated.
    pub target_id: String,
    /// Previous confidence score (if applicable).
    pub old_confidence: Option<f32>,
    /// New confidence score after update (if applicable).
    pub new_confidence: Option<f32>,
    /// Reason for the update.
    pub reason: String,
    /// Timestamp of the update.
    pub timestamp: i64,
}

impl LearningUpdate {
    /// Create a new learning update.
    pub fn new(
        action: LearningAction,
        target_kind: impl Into<String>,
        target_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            metadata: Metadata::new("learning_system"),
            action,
            target_kind: target_kind.into(),
            target_id: target_id.into(),
            old_confidence: None,
            new_confidence: None,
            reason: reason.into(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Set old confidence.
    pub fn with_old_confidence(mut self, confidence: f32) -> Self {
        self.old_confidence = Some(confidence);
        self
    }

    /// Set new confidence.
    pub fn with_new_confidence(mut self, confidence: f32) -> Self {
        self.new_confidence = Some(confidence);
        self
    }
}

impl Default for LearningUpdate {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            action: LearningAction::UpdateConfidence,
            target_kind: String::new(),
            target_id: String::new(),
            old_confidence: None,
            new_confidence: None,
            reason: String::new(),
            timestamp: 0,
        }
    }
}
