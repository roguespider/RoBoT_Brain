// src/database/models.rs

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==========================================================
// HYPOTHESIS ENGINE TYPES
// ==========================================================

/// Status of a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum HypothesisStatus {
    /// Hypothesis is being tested
    #[default]
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
impl std::fmt::Display for HypothesisStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HypothesisStatus::Testing => "testing",
            HypothesisStatus::Supported => "supported",
            HypothesisStatus::Refuted => "refuted",
            HypothesisStatus::Inconclusive => "inconclusive",
            HypothesisStatus::Superseded => "superseded",
        };
        write!(f, "{}", s)
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

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
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
        };
        write!(f, "{}", s)
    }
}

// ==========================================================
// MEMORY LAYER (STM vs LTM)
// ==========================================================

/// Memory layer per Architecture §6.3
/// - Working: Short-term, volatile, context-focused
/// - Permanent: Long-term, curated, indexed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum MemoryLayer {
    /// Short-term memory - temporary, high volatility
    #[default]
    Working,
    /// Long-term memory - curated, persistent, indexed
    Permanent,
}
impl std::fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemoryLayer::Working => "working",
            MemoryLayer::Permanent => "permanent",
        };
        write!(f, "{}", s)
    }
}

// ==========================================================
// HIERARCHY LEVELS
// ==========================================================

/// Level in the document hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum HierarchyLevel {
    /// Root level - whole document/file
    #[default]
    Document,
    /// Major section (h1, ## header, chapter)
    Section,
    /// Subsection (h2-h4, ### header)
    Subsection,
    /// Paragraph - natural text block
    Paragraph,
    /// Individual sentence (for fine-grained search)
    Sentence,
}
impl std::fmt::Display for HierarchyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            HierarchyLevel::Document => "document",
            HierarchyLevel::Section => "section",
            HierarchyLevel::Subsection => "subsection",
            HierarchyLevel::Paragraph => "paragraph",
            HierarchyLevel::Sentence => "sentence",
        };
        write!(f, "{}", s)
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

    // Memory layer - per Architecture §6.3
    pub layer: MemoryLayer,

    // Hierarchy fields for hierarchical storage
    pub parent_id: Option<Uuid>,           // None for root document
    pub hierarchy_level: HierarchyLevel,   // document, section, paragraph, sentence
    pub order_index: usize,               // Position within parent
    pub path: String,                     // e.g., "readme.md/section[0]/paragraph[2]"
    pub file_source: Option<String>,       // Original file path

    // Access tracking for consolidation
    pub access_count: u32,
    pub last_accessed: Option<DateTime<Utc>>,

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

            // Memory layer - starts in Working (STM)
            layer: MemoryLayer::Working,

            // Hierarchy fields - default for flat/non-hierarchical memory
            parent_id: None,
            hierarchy_level: HierarchyLevel::Document,
            order_index: 0,
            path: String::new(),
            file_source: None,

            // Access tracking
            access_count: 0,
            last_accessed: None,

            confidence: 0.5,

            importance: 0.5,

            created_at: now,

            updated_at: now,
        }
    }

    /// Create a new hierarchical memory card
    pub fn new_hierarchical(
        content: String,
        memory_type: MemoryType,
        parent_id: Option<Uuid>,
        hierarchy_level: HierarchyLevel,
        order_index: usize,
        path: String,
        file_source: Option<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            content,
            memory_type,
            layer: MemoryLayer::Working,
            parent_id,
            hierarchy_level,
            order_index,
            path,
            file_source,
            access_count: 0,
            last_accessed: None,
            confidence: 0.5,
            importance: 0.5,
            created_at: now,
            updated_at: now,
        }
    }
}

// ==========================================================
// MEMORY RELATIONSHIP
// ==========================================================

/// Relationship type between memories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[derive(Default)]
pub enum MemoryRelationshipType {
    /// General related relationship
    #[default]
    Related,
    /// Causal relationship (A causes B)
    Causes,
    /// Enables relationship (A enables B)
    Enables,
    /// Contradicts relationship (A contradicts B)
    Contradicts,
    /// Similar relationship (A is similar to B)
    Similar,
    /// Derived from relationship (A is derived from B)
    DerivedFrom,
}
impl std::fmt::Display for MemoryRelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemoryRelationshipType::Related => "related",
            MemoryRelationshipType::Causes => "causes",
            MemoryRelationshipType::Enables => "enables",
            MemoryRelationshipType::Contradicts => "contradicts",
            MemoryRelationshipType::Similar => "similar",
            MemoryRelationshipType::DerivedFrom => "derived_from",
        };
        write!(f, "{}", s)
    }
}

/// A relationship between two memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRelationship {
    /// Unique identifier
    pub id: Uuid,
    /// Source memory ID
    pub memory_id: Uuid,
    /// Target memory ID
    pub related_id: Uuid,
    /// Type of relationship
    pub relationship_type: MemoryRelationshipType,
}

impl MemoryRelationship {
    /// Create a new memory relationship
    pub fn new(memory_id: Uuid, related_id: Uuid, relationship_type: MemoryRelationshipType) -> Self {
        Self {
            id: Uuid::new_v4(),
            memory_id,
            related_id,
            relationship_type,
        }
    }
}

