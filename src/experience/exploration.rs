// /src/experience/exploration.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::ExperienceContext;

/// ============================================================================
/// EXPLORATION
/// ============================================================================

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

/// ============================================================================
/// STATUS
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExplorationStatus {
    Planned,
    Active,
    Paused,
    Completed,
    Abandoned,
}

/// ============================================================================
/// HYPOTHESIS
/// ============================================================================

/// A belief or assumption being tested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,

    /// The assumption being tested
    pub statement: String,

    /// Confidence before testing
    pub confidence: f32,

    /// Result after investigation
    pub result: Option<HypothesisResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypothesisResult {
    Supported,
    PartiallySupported,
    Rejected,
    Unknown,
}

/// ============================================================================
/// EXPLORATION ATTEMPT
/// ============================================================================

/// A single thing tried during exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationAttempt {
    pub id: String,

    pub timestamp: DateTime<Utc>,

    /// What was attempted
    pub action: String,

    /// Expected outcome
    pub expected_result: Option<String>,

    /// Actual outcome
    pub actual_result: Option<String>,

    /// Did this attempt succeed?
    pub success: bool,
}

/// ============================================================================
/// FINDINGS
/// ============================================================================

/// Knowledge gained from exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationFinding {
    pub id: String,

    pub timestamp: DateTime<Utc>,

    /// Observation or discovery
    pub description: String,

    /// Confidence in this finding
    pub confidence: f32,

    /// Whether this should become reusable knowledge
    pub promoted: bool,
}
