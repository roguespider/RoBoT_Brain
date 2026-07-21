// src/database/models.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

