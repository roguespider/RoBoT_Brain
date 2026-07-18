use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::experience::types::ExperienceContext;

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
