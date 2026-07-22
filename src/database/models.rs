// src/database/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==========================================================
// HYPOTHESIS ENGINE TYPES
// ==========================================================

/// Status of a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HypothesisStatus {
    /// Hypothesis is being tested
    Testing,
    /// Evidence supports the hypothesis
    Supported,
    /// Evidence contradicts the hypothesis
    Refuted,
    /// Not enough evidence yet
    Inconclusive,
    /// Superseded by a better hypothesis
    Superseded,
}

impl Default for HypothesisStatus {
    fn default() -> Self {
        HypothesisStatus::Testing
    }
}

impl ToString for HypothesisStatus {
    fn to_string(&self) -> String {
        match self {
            HypothesisStatus::Testing => "testing",
            HypothesisStatus::Supported => "supported",
            HypothesisStatus::Refuted => "refuted",
            HypothesisStatus::Inconclusive => "inconclusive",
            HypothesisStatus::Superseded => "superseded",
        }
        .to_string()
    }
}

/// A testable hypothesis derived from observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: Uuid,
    /// The hypothesis statement (e.g., "Using X approach improves Y outcome")
    pub statement: String,
    /// Category or domain (e.g., "workflow", "tool", "pattern")
    pub domain: String,
    /// Current status
    pub status: HypothesisStatus,
    /// Confidence level 0.0 - 1.0
    pub confidence: f32,
    /// Supporting evidence count
    pub supporting_count: u32,
    /// Contradicting evidence count
    pub contradicting_count: u32,
    /// Observations that led to this hypothesis
    pub source_observations: Vec<String>,
    /// Related memories/experiences
    pub related_memories: Vec<Uuid>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl Hypothesis {
    pub fn new(statement: String, domain: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            statement,
            domain,
            status: HypothesisStatus::Testing,
            confidence: 0.5,
            supporting_count: 0,
            contradicting_count: 0,
            source_observations: Vec::new(),
            related_memories: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// An observation that can trigger hypothesis formation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: Uuid,
    /// What was observed
    pub content: String,
    /// Context of the observation
    pub context: String,
    /// Type: success, failure, pattern, anomaly
    pub observation_type: String,
    /// Related experience IDs
    pub related_experiences: Vec<Uuid>,
    /// Whether this led to a hypothesis
    pub triggered_hypothesis: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Observation {
    pub fn new(content: String, context: String, observation_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            context,
            observation_type,
            related_experiences: Vec::new(),
            triggered_hypothesis: None,
            created_at: Utc::now(),
        }
    }
}

/// Evidence supporting or contradicting a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    /// The hypothesis this evidence belongs to
    pub hypothesis_id: Uuid,
    /// The evidence content
    pub content: String,
    /// Type: success, failure, correlation, anomaly
    pub evidence_type: String,
    /// Whether it supports or contradicts
    pub direction: String, // "support" or "contradict"
    /// Strength of evidence 0.0 - 1.0
    pub strength: f32,
    /// Related experience ID
    pub experience_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl Evidence {
    pub fn new(
        hypothesis_id: Uuid,
        content: String,
        evidence_type: String,
        direction: String,
        strength: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            hypothesis_id,
            content,
            evidence_type,
            direction,
            strength,
            experience_id: None,
            created_at: Utc::now(),
        }
    }
}

/// Knowledge extracted from validated hypotheses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Knowledge {
    pub id: Uuid,
    /// The learned knowledge/fact
    pub content: String,
    /// Source hypothesis
    pub source_hypothesis: Option<Uuid>,
    /// Confidence in this knowledge
    pub confidence: f32,
    /// Domain/category
    pub domain: String,
    /// How this was derived
    pub derivation: String,
    /// Whether it's active knowledge
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl Knowledge {
    pub fn new(content: String, domain: String, derivation: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            source_hypothesis: None,
            confidence: 0.5,
            domain,
            derivation,
            active: true,
            created_at: Utc::now(),
        }
    }
}

// ==========================================================
// MEMORY TYPES
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum MemoryType {
    #[default]
    Note,
    Fact,
    Task,
    File,
    Conversation,
    Code,
    Decision,
    Event,
    Encounter,
    Experience,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for MemoryType {
    fn to_string(&self) -> String {
        match self {
            MemoryType::Note => "note",
            MemoryType::Fact => "fact",
            MemoryType::Task => "task",
            MemoryType::File => "file",
            MemoryType::Conversation => "conversation",
            MemoryType::Code => "code",
            MemoryType::Decision => "decision",
            MemoryType::Event => "event",
            MemoryType::Encounter => "encounter",
            MemoryType::Experience => "experience",
        }
        .to_string()
    }
}

// ==========================================================
// CORE MEMORY CARD
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCard {
    pub id: Uuid,

    pub content: String,

    pub memory_type: MemoryType,

    pub confidence: f32,

    pub importance: f32,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,
}

impl MemoryCard {
    pub fn new(content: String, memory_type: MemoryType) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),

            content,

            memory_type,

            confidence: 0.5,

            importance: 0.5,

            created_at: now,

            updated_at: now,
        }
    }
}

// ==========================================================
// MEMORY SOURCE
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySource {
    pub id: Uuid,

    pub memory_id: Uuid,

    /// Examples:
    /// chat_export
    /// imported_file
    /// user_input
    /// generated
    pub source_type: String,

    pub source_name: String,

    pub source_location: Option<String>,

    pub created_at: DateTime<Utc>,
}

// ==========================================================
// GRAPH RELATIONSHIP
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,

    /// Starting memory.
    pub source_id: Uuid,

    /// Connected memory.
    pub target_id: Uuid,

    /// Example:
    /// caused_by
    /// related_to
    /// depends_on
    /// part_of
    pub relationship: String,

    pub strength: f32,

    pub created_at: DateTime<Utc>,
}

// ==========================================================
// DECISION MEMORY
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: Uuid,

    /// Problem being solved.
    pub task: String,

    /// Selected method.
    pub chosen_workflow: String,

    /// Alternatives considered.
    pub alternatives: Vec<String>,

    /// Reasoning behind choice.
    pub reasoning: String,

    /// Result after execution.
    pub result: Option<String>,

    /// Whether it succeeded.
    pub success: Option<bool>,

    /// Confidence in decision.
    pub confidence: f32,

    pub created_at: DateTime<Utc>,
}

impl DecisionRecord {
    pub fn new(task: String, workflow: String, reasoning: String) -> Self {
        Self {
            id: Uuid::new_v4(),

            task,

            chosen_workflow: workflow,

            alternatives: Vec::new(),

            reasoning,

            result: None,

            success: None,

            confidence: 0.5,

            created_at: Utc::now(),
        }
    }
}

// ==========================================================
// EVENT LOG
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub id: Uuid,

    /// Example:
    /// memory_created
    /// file_imported
    /// workflow_completed
    pub event_type: String,

    pub description: String,

    /// Optional link to memory/decision/file.
    pub related_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
}

