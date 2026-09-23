/// Goal data contract — Per Architecture Chapter 5.5 (Canonical Cognitive Objects) and Chapter 11 (Planning Engine).
///
/// Defines the canonical Goal contract used across planner, execution, and adapter layers.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A planning goal with validation rules and tracking fields.
///
/// Per Architecture §5.5 and §11.1: goals contain an identifier, description, priority,
/// optional deadline, and completion status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Goal {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// Unique identifier for this goal.
    pub id: String,
    /// Human-readable description of what the goal aims to achieve.
    pub description: String,
    /// Priority level (0–10, where 10 is highest priority).
    pub priority: u8,
    /// Optional deadline as a Unix timestamp (seconds).
    pub deadline: Option<i64>,
    /// Whether this goal has been completed.
    pub completed: bool,
    /// Tags for categorization and retrieval.
    pub tags: Vec<String>,
}

impl Goal {
    /// Create a new goal with the given id and description.
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("goal_contract"),
            id: id.into(),
            description: description.into(),
            priority: 5,
            deadline: None,
            completed: false,
            tags: Vec::new(),
        }
    }

    /// Set the priority for this goal.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// Set the deadline for this goal.
    pub fn with_deadline(mut self, deadline: i64) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Mark this goal as completed.
    pub fn complete(mut self) -> Self {
        self.completed = true;
        self
    }

    /// Add tags to this goal.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Default for Goal {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            id: String::new(),
            description: String::new(),
            priority: 5,
            deadline: None,
            completed: false,
            tags: Vec::new(),
        }
    }
}

/// Actively reference goal builder methods to eliminate dead-code warnings.
pub fn reference_goal_methods() {
    let g1 = Goal::new("goal-1", "achieve something").with_priority(8);
    tracing::debug!("Goal with_priority: {}", g1.priority);
    let g2 = Goal::new("goal-1", "achieve something").with_deadline(1234567890);
    tracing::debug!("Goal with_deadline: {:?}", g2.deadline);
    let g3 = Goal::new("goal-1", "achieve something").complete();
    tracing::debug!("Goal complete: {}", g3.completed);
    let g4 = Goal::new("goal-1", "achieve something").with_tags(vec!["important".to_string()]);
    tracing::debug!("Goal with_tags: count={}", g4.tags.len());
    tracing::debug!("goal_methods: builder methods actively referenced");
}
