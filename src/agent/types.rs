// src/agent/types.rs
//! Goal and status types for the agent loop (Architecture §5.7).

use serde::{Deserialize, Serialize};

/// Identifier for a goal the agent pursues.
pub type AgentGoalId = String;

/// Lifecycle status of a goal within the agent loop.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalStatus {
    /// Goal accepted but planning not yet started.
    Pending,
    /// A plan was produced; actions are being selected/executed.
    InProgress,
    /// Goal achieved; outcome recorded as a successful experience.
    Achieved,
    /// Goal could not be achieved; outcome recorded as a failed experience.
    Failed,
    /// Agent declined to act (safety gate blocked, or confidence too low).
    Abstained,
}

/// A goal the agent pursues through the cognitive loop.
///
/// Per Architecture §5.7 Decision Flow, a goal is decomposed into a plan, the
/// plan's first actionable step is selected, supporting memory/knowledge/
/// experience are retrieved, confidence is evaluated, and — if above threshold
/// and not safety-blocked — the action is executed and its outcome recorded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGoal {
    pub id: AgentGoalId,
    /// Human-readable description of what the agent should accomplish.
    pub description: String,
    /// Minimum confidence required to act (Architecture §5.7 confidence gate).
    pub confidence_threshold: f32,
    pub status: GoalStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl AgentGoal {
    /// Create a new pending goal with a sensible default confidence threshold.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            confidence_threshold: 0.5,
            status: GoalStatus::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    /// Create a goal with a custom confidence threshold.
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }
}
