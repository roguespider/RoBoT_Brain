//! Exploration system types and the main Exploration struct.
//!
//! Per Architecture §2.7:
//! - Exploration is intentional investigation performed by the system
//! - Exploration records the journey toward understanding
//! - Exploration is not a decision
//!
//! ## Module Structure
//! - `exploration.rs` - Main Exploration struct and ExplorationStatus enum
//! - `attempt.rs` - ExplorationAttempt struct for recording attempts
//! - `finding.rs` - ExplorationFinding struct for discoveries
//! - `hypothesis.rs` - Hypothesis struct and HypothesisResult enum
//! - `store.rs` - ExplorationRepository trait for persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::experience::types::ExperienceContext;

// Import types from sibling modules
use super::attempt::ExplorationAttempt;
use super::finding::ExplorationFinding;
use super::hypothesis::Hypothesis;

/// Represents an intentional investigation performed by the system.
///
/// Exploration is not a decision.
/// It records the journey toward understanding.
///
/// Per Architecture §2.7:
/// - `title` - Human readable identifier
/// - `purpose` - Why this exploration exists
/// - `hypotheses` - Initial assumptions before investigation
/// - `attempts` - Things tried during exploration
/// - `findings` - What was learned
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

impl Exploration {
    /// Create a new exploration with initial values.
    pub fn new(id: String, title: String, purpose: String, context: ExperienceContext) -> Self {
        Self {
            id,
            started_at: Utc::now(),
            completed_at: None,
            title,
            purpose,
            status: ExplorationStatus::Planned,
            context,
            hypotheses: Vec::new(),
            attempts: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Start the exploration (transition from Planned to Active).
    pub fn start(&mut self) {
        self.status = ExplorationStatus::Active;
    }

    /// Pause the exploration.
    pub fn pause(&mut self) {
        self.status = ExplorationStatus::Paused;
    }

    /// Complete the exploration successfully.
    pub fn complete(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = ExplorationStatus::Completed;
    }

    /// Abandon the exploration.
    pub fn abandon(&mut self) {
        self.completed_at = Some(Utc::now());
        self.status = ExplorationStatus::Abandoned;
    }

    /// Add a hypothesis to test.
    pub fn add_hypothesis(&mut self, hypothesis: Hypothesis) {
        self.hypotheses.push(hypothesis);
    }

    /// Add an attempt made during exploration.
    pub fn add_attempt(&mut self, attempt: ExplorationAttempt) {
        self.attempts.push(attempt);
    }

    /// Add a finding from exploration.
    pub fn add_finding(&mut self, finding: ExplorationFinding) {
        self.findings.push(finding);
    }

    /// Check if exploration is still active.
    pub fn is_active(&self) -> bool {
        self.status == ExplorationStatus::Active
    }

    /// Check if exploration has completed (successfully or abandoned).
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            ExplorationStatus::Completed | ExplorationStatus::Abandoned
        )
    }
}

/// Current exploration state.
///
/// Per Architecture §2.7, exploration follows a lifecycle:
/// - Planned → Active → Completed/Abandoned
/// - Can be paused mid-exploration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExplorationStatus {
    /// Exploration is planned but not yet started
    Planned,
    /// Exploration is actively underway
    Active,
    /// Exploration is temporarily paused
    Paused,
    /// Exploration completed successfully
    Completed,
    /// Exploration was abandoned without completion
    Abandoned,
}
