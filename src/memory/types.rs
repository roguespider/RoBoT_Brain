// src/memory/types.rs

//! Memory types - Per Architecture §6.3

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Memory layer type - Per Architecture Chapter 08.
/// All 9 layers: Working, Short-Term, Long-Term, Semantic, Episodic,
/// Procedural, Graph, Index, Archive.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum MemoryLayer {
    /// Working Memory - Temporary information during active reasoning
    #[default]
    Working,
    /// Short-Term Memory - Bridges nearby conversations
    ShortTerm,
    /// Long-Term Memory - Durable learned knowledge
    LongTerm,
    /// Semantic Memory - Facts independent of specific events
    Semantic,
    /// Episodic Memory - Specific experiences and events
    Episodic,
    /// Procedural Memory - How to perform tasks
    Procedural,
    /// Graph Memory - Connected knowledge with relationships
    Graph,
    /// Index Memory - Symbolic indexes for fast lookup (layer missing in prior code)
    Index,
    /// Archive Memory - Old/low-value items preserved but inactive
    Archive,
}

impl std::fmt::Display for MemoryLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryLayer::Working => write!(f, "Working"),
            MemoryLayer::ShortTerm => write!(f, "ShortTerm"),
            MemoryLayer::LongTerm => write!(f, "LongTerm"),
            MemoryLayer::Semantic => write!(f, "Semantic"),
            MemoryLayer::Episodic => write!(f, "Episodic"),
            MemoryLayer::Procedural => write!(f, "Procedural"),
            MemoryLayer::Graph => write!(f, "Graph"),
            MemoryLayer::Index => write!(f, "Index"),
            MemoryLayer::Archive => write!(f, "Archive"),
        }
    }
}

impl MemoryLayer {
    /// Return the default layer for newly observed memories.
    pub fn default_for_observation() -> Self {
        Self::Working
    }

    /// Check if this layer supports permanent storage.
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::LongTerm
                | Self::Semantic
                | Self::Episodic
                | Self::Procedural
                | Self::Graph
                | Self::Index
                | Self::Archive
        )
    }

    /// Check if this layer supports graph-based retrieval.
    pub fn supports_graph(&self) -> bool {
        matches!(self, Self::Graph | Self::Semantic | Self::Episodic)
    }

    /// Check if this layer supports symbolic/index lookup.
    pub fn supports_index(&self) -> bool {
        matches!(self, Self::Index | Self::Semantic | Self::Procedural)
    }

    /// Return the promotion target for this layer, if any.
    /// Per Architecture Chapter 08: Working→ShortTerm→LongTerm→Semantic→Episodic→Procedural→Graph→Index→Archive.
    pub fn promote_to(&self) -> Option<MemoryLayer> {
        match self {
            MemoryLayer::Working => Some(MemoryLayer::ShortTerm),
            MemoryLayer::ShortTerm => Some(MemoryLayer::LongTerm),
            MemoryLayer::LongTerm => Some(MemoryLayer::Semantic),
            MemoryLayer::Semantic => Some(MemoryLayer::Episodic),
            MemoryLayer::Episodic => Some(MemoryLayer::Procedural),
            MemoryLayer::Procedural => Some(MemoryLayer::Graph),
            MemoryLayer::Graph => Some(MemoryLayer::Index),
            MemoryLayer::Index => Some(MemoryLayer::Archive),
            MemoryLayer::Archive => None,
        }
    }

    /// Return the demotion target for this layer, if any.
    pub fn demote_to(&self) -> Option<MemoryLayer> {
        match self {
            MemoryLayer::ShortTerm => Some(MemoryLayer::Working),
            MemoryLayer::LongTerm => Some(MemoryLayer::ShortTerm),
            MemoryLayer::Semantic => Some(MemoryLayer::LongTerm),
            MemoryLayer::Episodic => Some(MemoryLayer::Semantic),
            MemoryLayer::Procedural => Some(MemoryLayer::Episodic),
            MemoryLayer::Graph => Some(MemoryLayer::Procedural),
            MemoryLayer::Index => Some(MemoryLayer::Graph),
            MemoryLayer::Archive => Some(MemoryLayer::Index),
            MemoryLayer::Working => None,
        }
    }

    /// Return a human-readable name for the layer.
    pub fn name(&self) -> &'static str {
        match self {
            MemoryLayer::Working => "working",
            MemoryLayer::ShortTerm => "short_term",
            MemoryLayer::LongTerm => "long_term",
            MemoryLayer::Semantic => "semantic",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Procedural => "procedural",
            MemoryLayer::Graph => "graph",
            MemoryLayer::Index => "index",
            MemoryLayer::Archive => "archive",
        }
    }
}

/// Memory item type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Experience-based memory
    Experience,
    /// Concept stored as knowledge
    Concept,
    /// Knowledge fact
    Knowledge,
    /// Procedural skill
    Skill,
    /// Workflow definition
    Workflow,
    /// User context
    Context,
    /// Observation record
    Observation,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemoryType::Experience => "experience",
            MemoryType::Concept => "concept",
            MemoryType::Knowledge => "knowledge",
            MemoryType::Skill => "skill",
            MemoryType::Workflow => "workflow",
            MemoryType::Context => "context",
            MemoryType::Observation => "observation",
        };
        write!(f, "{}", s)
    }
}

/// Memory status - Per Architecture §6.3
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum MemoryStatus {
    /// Active and accessible
    #[default]
    Active,
    /// Inactive but retained
    Inactive,
    /// Archived for later retrieval
    Archived,
    /// Being garbage collected
    PendingDeletion,
}

/// Provenance tracking for research results promoted to memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProvenance {
    pub url: String,
    pub provider: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub query: String,
}

#[derive(Debug, Clone)]
pub enum MemoryError {
    InsufficientConfidence,
    StorageFailed,
}

/// A memory item - Per Architecture §6.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier
    pub id: Uuid,

    /// Memory layer (Working or Permanent)
    pub layer: MemoryLayer,

    /// Memory type classification
    pub memory_type: MemoryType,

    /// Status of this memory
    pub status: MemoryStatus,

    /// Content/description of this memory
    pub content: String,

    /// Optional title (MemoryRecord contract, Architecture §5.6).
    pub title: Option<String>,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,

    /// Importance level (0.0 - 1.0)
    pub importance: f32,

    /// When this memory was created
    pub created_at: DateTime<Utc>,

    /// When this memory was last accessed
    pub accessed_at: DateTime<Utc>,

    /// When this memory was last modified
    pub modified_at: DateTime<Utc>,

    /// When this memory was last consolidated (promoted to permanent)
    pub last_consolidated: Option<DateTime<Utc>>,

    /// Access count
    pub access_count: u32,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Source of this memory (experience, user, system)
    pub source: String,

    /// Memory lifecycle state (Working, Candidate, Accepted, Permanent, Archived)
    pub lifecycle: crate::memory::lifecycle::MemoryLifecycle,

    /// Related memory IDs
    pub related_ids: Vec<Uuid>,
}

impl MemoryItem {
    /// Create a new memory item
    pub fn new(
        layer: MemoryLayer,
        memory_type: MemoryType,
        content: String,
        source: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            layer,
            memory_type,
            status: MemoryStatus::Active,
            content,
            title: None,
            confidence: 0.5,
            importance: 0.5,
            created_at: now,
            accessed_at: now,
            modified_at: now,
            last_consolidated: None,
            lifecycle: crate::memory::lifecycle::MemoryLifecycle::Working,
            access_count: 0,
            tags: Vec::new(),
            source,
            related_ids: Vec::new(),
        }
    }

    /// Record an access to this memory
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Utc::now();
    }

    /// Archive this memory
    pub fn archive(&mut self) {
        self.status = MemoryStatus::Archived;
        self.lifecycle = crate::memory::lifecycle::MemoryLifecycle::Archived;
        self.modified_at = Utc::now();
    }

    /// Promote this memory to the next lifecycle state
    pub fn promote(&mut self) -> bool {
        let next = match self.lifecycle {
            crate::memory::lifecycle::MemoryLifecycle::Working => {
                Some(crate::memory::lifecycle::MemoryLifecycle::Candidate)
            }
            crate::memory::lifecycle::MemoryLifecycle::Candidate => {
                Some(crate::memory::lifecycle::MemoryLifecycle::Accepted)
            }
            crate::memory::lifecycle::MemoryLifecycle::Accepted => {
                Some(crate::memory::lifecycle::MemoryLifecycle::Permanent)
            }
            crate::memory::lifecycle::MemoryLifecycle::Permanent => None,
            crate::memory::lifecycle::MemoryLifecycle::Archived => None,
        };
        match next {
            Some(state) => {
                self.lifecycle = state;
                self.modified_at = Utc::now();
                if state == crate::memory::lifecycle::MemoryLifecycle::Permanent {
                    self.last_consolidated = Some(Utc::now());
                    self.layer = MemoryLayer::LongTerm;
                }
                true
            }
            None => false,
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

// ============================================================
// CONVERSION TO/FROM DATABASE MODELS
// ============================================================

use crate::database::models::{HierarchyLevel, MemoryCard};

impl From<&MemoryCard> for MemoryItem {
    fn from(card: &MemoryCard) -> Self {
        Self {
            id: card.id,
            layer: match card.layer {
                MemoryLayer::Working => MemoryLayer::Working,
                MemoryLayer::ShortTerm => MemoryLayer::ShortTerm,
                MemoryLayer::LongTerm => MemoryLayer::LongTerm,
                MemoryLayer::Semantic => MemoryLayer::Semantic,
                MemoryLayer::Episodic => MemoryLayer::Episodic,
                MemoryLayer::Procedural => MemoryLayer::Procedural,
                MemoryLayer::Graph => MemoryLayer::Graph,
                MemoryLayer::Index => MemoryLayer::Index,
                MemoryLayer::Archive => MemoryLayer::Archive,
            },
            memory_type: match card.memory_type {
                crate::database::models::MemoryType::Note => MemoryType::Experience,
                crate::database::models::MemoryType::Fact => MemoryType::Knowledge,
                crate::database::models::MemoryType::Concept => MemoryType::Concept,
                crate::database::models::MemoryType::Task => MemoryType::Skill,
                crate::database::models::MemoryType::File => MemoryType::Workflow,
                crate::database::models::MemoryType::Conversation => MemoryType::Context,
                crate::database::models::MemoryType::Code => MemoryType::Skill,
                crate::database::models::MemoryType::Decision => MemoryType::Experience,
                crate::database::models::MemoryType::Event => MemoryType::Observation,
                crate::database::models::MemoryType::Encounter => MemoryType::Observation,
                crate::database::models::MemoryType::Experience => MemoryType::Experience,
            },
            status: MemoryStatus::Active,
            content: card.content.clone(),
            title: card.title.clone(),
            confidence: card.confidence,
            importance: card.importance,
            created_at: card.created_at,
            accessed_at: card.last_accessed.unwrap_or(card.created_at),
            modified_at: card.updated_at,
            last_consolidated: None,
            lifecycle: crate::memory::lifecycle::MemoryLifecycle::Working,
            access_count: card.access_count,
            tags: Vec::new(),
            source: card
                .file_source
                .clone()
                .unwrap_or_else(|| "database".to_string()),
            related_ids: Vec::new(),
        }
    }
}

impl From<MemoryItem> for MemoryCard {
    fn from(item: MemoryItem) -> Self {
        Self {
            id: item.id,
            content: item.content,
            title: item.title,
            memory_type: match item.memory_type {
                MemoryType::Experience => crate::database::models::MemoryType::Experience,
                MemoryType::Knowledge => crate::database::models::MemoryType::Fact,
                MemoryType::Concept => crate::database::models::MemoryType::Concept,
                MemoryType::Skill => crate::database::models::MemoryType::Code,
                MemoryType::Workflow => crate::database::models::MemoryType::File,
                MemoryType::Context => crate::database::models::MemoryType::Conversation,
                MemoryType::Observation => crate::database::models::MemoryType::Event,
            },
            layer: match item.layer {
                MemoryLayer::Working => MemoryLayer::Working,
                MemoryLayer::ShortTerm => MemoryLayer::ShortTerm,
                MemoryLayer::LongTerm => MemoryLayer::LongTerm,
                MemoryLayer::Semantic => MemoryLayer::Semantic,
                MemoryLayer::Episodic => MemoryLayer::Episodic,
                MemoryLayer::Procedural => MemoryLayer::Procedural,
                MemoryLayer::Graph => MemoryLayer::Graph,
                MemoryLayer::Index => MemoryLayer::Index,
                MemoryLayer::Archive => MemoryLayer::Archive,
            },
            parent_id: None,
            hierarchy_level: HierarchyLevel::Document,
            order_index: 0,
            path: String::new(),
            file_source: Some(item.source).filter(|s| s != "database"),
            access_count: item.access_count,
            last_accessed: Some(item.accessed_at),
            confidence: item.confidence,
            importance: item.importance,
            created_at: item.created_at,
            updated_at: item.modified_at,
        }
    }
}
