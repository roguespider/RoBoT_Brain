// /src/experience/types.rs

// Part 1: Imports, Experience, ExperienceType, ExperienceContext, Evidence, ExperienceSource
// Part 2: ExperienceOutcome, KnowledgeMaturity, Encounter, EncounterStats
// Part 3: ExperienceScore, ImportanceLevel, ReputationTarget, ReputationRecord, plus the closing of the file

// src/experience/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// ============================================================================
/// EXPERIENCE
/// ============================================================================

/// A single recorded experience within the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Unique identifier.
    pub id: Uuid,

    /// When the experience occurred.
    pub timestamp: DateTime<Utc>,

    /// Category of experience.
    pub experience_type: ExperienceType,

    /// Human-readable title.
    pub title: String,

    /// Detailed description.
    pub description: String,

    /// Context surrounding the experience.
    pub context: ExperienceContext,

    /// Outcome of the experience.
    pub outcome: ExperienceOutcome,

    /// Calculated later by scorer.rs
    pub score: Option<ExperienceScore>,

    /// Encounters contributing to this experience.
    pub encounter_ids: Vec<Uuid>,

    /// Current maturity level.
    pub maturity: KnowledgeMaturity,

    /// Overall confidence.
    pub confidence: f32,

    /// Lessons learned.
    pub lessons: Vec<String>,

    /// Supporting evidence count.
    pub evidence_count: usize,

    /// Searchable tags.
    pub tags: Vec<String>,

    /// Arbitrary metadata.
    pub metadata: HashMap<String, String>,
}

/// Categories of experiences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceType {
    ToolExecution,
    MemoryLookup,
    MemoryStore,
    Workflow,
    Planning,
    Exploration,
    Hypothesis,
    Reflection,
    Learning,
    Conversation,
    UserFeedback,
    ModelInference,
    Error,
    System,
    Custom(String),
}

/// ============================================================================
/// EXPERIENCE CONTEXT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExperienceContext {
    pub workflow: Option<WorkflowContext>,
    pub tool: Option<ToolContext>,
    pub model: Option<ModelContext>,

    pub session_id: Option<String>,
    pub parent_experience: Option<Uuid>,
    pub user_query: Option<String>,
}

/// Workflow information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub name: String,
    pub step: Option<String>,
    pub parent_workflow: Option<String>,
}

/// Tool information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub name: String,
    pub version: Option<String>,
    pub arguments: HashMap<String, String>,
}

/// Model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContext {
    pub name: String,
    pub provider: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// ============================================================================
/// EVIDENCE
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Evidence identifier.
    pub id: Uuid,

    /// Experiences supported by this evidence.
    pub experience_ids: Vec<Uuid>,

    /// Confidence in this evidence.
    pub confidence: f32,
}

/// ============================================================================
/// EXPERIENCE SOURCE
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceSource {
    User,
    Tool,
    Planner,
    Memory,
    Reflection,
    Exploration,
    Hypothesis,
    Evolution,
    System,
    Model,
}

/// ============================================================================
/// EXPERIENCE OUTCOME
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceOutcome {
    /// Overall outcome.
    pub kind: OutcomeKind,

    /// Optional informational message.
    pub message: Option<String>,

    /// Optional error message.
    pub error: Option<String>,

    /// Execution duration in milliseconds.
    pub duration_ms: Option<u64>,
}

impl ExperienceOutcome {
    /// Successful execution.
    pub fn success() -> Self {
        Self {
            kind: OutcomeKind::Success,
            message: None,
            error: None,
            duration_ms: None,
        }
    }

    /// Partially successful execution.
    pub fn partial(message: impl Into<String>) -> Self {
        Self {
            kind: OutcomeKind::Partial,
            message: Some(message.into()),
            error: None,
            duration_ms: None,
        }
    }

    /// Failed execution.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            kind: OutcomeKind::Failure,
            message: None,
            error: Some(error.into()),
            duration_ms: None,
        }
    }

    /// Timed out.
    pub fn timeout() -> Self {
        Self {
            kind: OutcomeKind::Timeout,
            message: None,
            error: None,
            duration_ms: None,
        }
    }

    /// Interrupted before completion.
    pub fn interrupted() -> Self {
        Self {
            kind: OutcomeKind::Interrupted,
            message: None,
            error: None,
            duration_ms: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutcomeKind {
    Success,
    Failure,
    Partial,
    Timeout,
    Interrupted,
}

/// ============================================================================
/// KNOWLEDGE MATURITY
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeMaturity {
    /// Newly discovered.
    Emerging,

    /// Some supporting evidence exists.
    Developing,

    /// Repeatedly confirmed.
    Established,

    /// Highly trusted over time.
    Trusted,

    /// Confidence is decreasing.
    Questioned,

    /// Replaced by better information.
    Deprecated,

    /// Proven incorrect.
    Rejected,
}

/// ============================================================================
/// ENCOUNTER
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    /// Unique encounter identifier.
    pub id: Uuid,

    /// When the encounter occurred.
    pub timestamp: DateTime<Utc>,

    /// Related experience.
    pub experience_id: Option<Uuid>,

    /// Context surrounding the encounter.
    pub context: ExperienceContext,

    /// Original input.
    pub input: String,

    /// Action performed.
    pub action: String,

    /// Result of the encounter.
    pub result: EncounterResult,

    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncounterResult {
    Success,
    Failure,

    /// Partial completion with explanation.
    Partial(String),

    /// Error message.
    Error(String),

    Timeout,
}

/// ============================================================================
/// ENCOUNTER STATISTICS
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterStats {
    /// Experience being tracked.
    pub experience_id: Uuid,

    /// Total encounters.
    pub total_encounters: u64,

    /// Successful encounters.
    pub successes: u64,

    /// Failed encounters.
    pub failures: u64,

    /// First observed.
    pub first_seen: DateTime<Utc>,

    /// Most recent observation.
    pub last_seen: DateTime<Utc>,

    /// Average calculated score.
    pub average_score: f32,
}

/// ============================================================================
/// EXPERIENCE SCORING
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceScore {
    /// Overall importance of the experience.
    pub importance: f32,

    /// Confidence in the recorded outcome.
    pub confidence: f32,

    /// How different this experience is from previous ones.
    pub novelty: f32,

    /// Long-term reliability.
    pub reliability: f32,
}

impl Default for ExperienceScore {
    fn default() -> Self {
        Self {
            importance: 0.0,
            confidence: 0.0,
            novelty: 0.0,
            reliability: 0.0,
        }
    }
}

/// Relative importance assigned by the scoring engine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportanceLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// ============================================================================
/// REPUTATION
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationTarget {
    Tool(String),
    Workflow(String),
    Memory(String),
    Model(String),
    Hypothesis(Uuid),
    Exploration(Uuid),
    Experience(Uuid),
    Agent(String),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationRecord {
    /// Entity whose reputation is being tracked.
    pub target: ReputationTarget,

    /// Overall reputation score (typically 0.0..1.0).
    pub score: f32,

    /// Successful outcomes.
    pub successes: u64,

    /// Failed outcomes.
    pub failures: u64,

    /// Number of observations.
    pub observations: u64,

    /// Average confidence across observations.
    pub confidence: f32,

    /// Last update time.
    pub last_updated: DateTime<Utc>,
}

impl ReputationRecord {
    /// Create a new reputation record.
    pub fn new(target: ReputationTarget) -> Self {
        Self {
            target,
            score: 0.0,
            successes: 0,
            failures: 0,
            observations: 0,
            confidence: 0.0,
            last_updated: Utc::now(),
        }
    }

    /// Record a successful observation.
    pub fn record_success(&mut self, confidence: f32) {
        self.successes += 1;
        self.observations += 1;
        self.confidence = confidence;
        self.last_updated = Utc::now();
    }

    /// Record a failed observation.
    pub fn record_failure(&mut self, confidence: f32) {
        self.failures += 1;
        self.observations += 1;
        self.confidence = confidence;
        self.last_updated = Utc::now();
    }
}

