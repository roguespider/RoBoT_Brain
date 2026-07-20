use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::experience::types::ExperienceContext;

/// Represents an exploration attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationAttempt {
    /// What was attempted
    pub description: String,
    /// When it happened
    pub timestamp: DateTime<Utc>,
    /// Whether it succeeded
    pub success: bool,
}

/// Represents an exploration finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationFinding {
    /// What was discovered
    pub description: String,
    /// How confident we are (0.0 - 1.0)
    pub confidence: f32,
}

/// Represents an exploration hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// The hypothesis statement
    pub statement: String,
    /// Initial confidence (0.0 - 1.0)
    pub initial_confidence: f32,
}

/// Represents an intentional investigation performed by the system.
///
/// Exploration is not a decision.
/// It records the journey toward understanding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exploration {
    /// Unique exploration identifier
    pub id: String,

    /// When exploration began
    pub started_at: DateTime<Utc>,

    /// When exploration completed
    pub completed_at: Option<DateTime<Utc>>,

    /// Human readable title
    pub title: String,

    /// Why this exploration exists
    pub purpose: String,

    /// Current exploration state
    pub status: ExplorationStatus,

    /// Environment or situation where exploration happened
    pub context: ExperienceContext,

    /// Initial assumptions before investigation
    pub hypotheses: Vec<Hypothesis>,

    /// Attempts made during exploration
    pub attempts: Vec<ExplorationAttempt>,

    /// What was learned
    pub findings: Vec<ExplorationFinding>,
}

/// Current exploration state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExplorationStatus {
    Planned,
    Active,
    Paused,
    Completed,
    Abandoned,
}
