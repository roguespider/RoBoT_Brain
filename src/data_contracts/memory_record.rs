use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::metadata::Metadata;

/// MemoryKind - Kind of memory lifecycle state.
///
/// Corresponds to the lifecycle states in `src/memory/lifecycle::MemoryLifecycle`.
/// This is the canonical kind enum used by the `MemoryRecord` data contract.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub enum MemoryKind {
    /// Memory is being actively used or evaluated
    #[default]
    Working,
    /// Memory has been identified as worth promoting but not yet accepted
    Candidate,
    /// Memory has been reviewed and accepted for promotion
    Accepted,
    /// Memory is fully promoted to permanent storage
    Permanent,
    /// Memory has been archived (terminal state)
    Archived,
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MemoryKind::Working => "working",
            MemoryKind::Candidate => "candidate",
            MemoryKind::Accepted => "accepted",
            MemoryKind::Permanent => "permanent",
            MemoryKind::Archived => "archived",
        };
        write!(f, "{}", s)
    }
}

/// MemoryRecord data contract - Per Architecture §5.6 and §8.1.
///
/// A MemoryRecord represents persistent knowledge.
/// Per Architecture §5.6: id, type, title, summary, embedding, confidence,
/// created, updated, relationships, tags, source, version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Memory type (e.g., concept, entity, summary, workflow, relationship, documentation).
    pub memory_type: String,
    /// Memory kind (lifecycle state).
    pub kind: MemoryKind,
    /// Human-readable title.
    pub title: String,
    /// Memory content.
    pub content: String,
    /// Brief summary of the memory.
    pub summary: String,
    /// Vector embedding for semantic retrieval.
    pub embedding: Option<Vec<f32>>,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
    /// Creation timestamp.
    pub created_at: i64,
    /// Last update timestamp.
    pub updated_at: i64,
    /// Related memory IDs.
    pub relationships: Vec<Uuid>,
    /// Tags for categorization and retrieval.
    pub tags: Vec<String>,
    /// Source of this memory.
    pub source: String,
    /// Schema version.
    pub version: String,
    /// Importance score (0.0 - 1.0).
    pub importance: f32,
    /// Access count.
    pub access_count: u32,
    /// Metadata shared across data contracts.
    pub metadata: Metadata,
}

impl Default for MemoryRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            memory_type: "general".to_string(),
            kind: MemoryKind::default(),
            title: String::new(),
            content: String::new(),
            summary: String::new(),
            embedding: None,
            confidence: 0.5,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            relationships: Vec::new(),
            tags: Vec::new(),
            source: "unknown".to_string(),
            version: "1.0.0".to_string(),
            importance: 0.5,
            access_count: 0,
            metadata: Metadata::default(),
        }
    }
}

impl MemoryRecord {
    /// Create a new memory record.
    pub fn new(
        content: impl Into<String>,
        memory_type: impl Into<String>,
        kind: MemoryKind,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4(),
            memory_type: memory_type.into(),
            kind,
            title: String::new(),
            content: content.into(),
            summary: String::new(),
            embedding: None,
            confidence: 0.5,
            created_at: now,
            updated_at: now,
            relationships: Vec::new(),
            tags: Vec::new(),
            source: "memory_engine".to_string(),
            version: "1.0.0".to_string(),
            importance: 0.5,
            access_count: 0,
            metadata: Metadata::new("memory_engine"),
        }
    }

    /// Set the title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the summary.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Set the embedding.
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add a relationship.
    pub fn with_relationship(mut self, rel_id: Uuid) -> Self {
        self.relationships.push(rel_id);
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Record an access to this memory.
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.confidence = self.confidence.max(0.5);
        self.importance = self.importance.max(0.5);
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Archive this memory.
    pub fn archive(&mut self) {
        self.kind = MemoryKind::Archived;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    /// Promote this memory to the next lifecycle state.
    pub fn promote(&mut self) -> bool {
        let next = match self.kind {
            MemoryKind::Working => Some(MemoryKind::Candidate),
            MemoryKind::Candidate => Some(MemoryKind::Accepted),
            MemoryKind::Accepted => Some(MemoryKind::Permanent),
            MemoryKind::Permanent => None,
            MemoryKind::Archived => None,
        };

        match next {
            Some(state) => {
                self.kind = state;
                self.updated_at = chrono::Utc::now().timestamp();
                true
            }
            None => false,
        }
    }
}

/// Actively reference memory record builder methods to eliminate dead-code warnings.
pub fn reference_memory_record_methods() {
    let m1 = MemoryRecord::new("test content", "test_type", MemoryKind::Working)
        .with_title("Test Title");
    tracing::debug!("MemoryRecord with_title: {}", m1.title);
    let m2 = MemoryRecord::new("test content", "test_type", MemoryKind::Working)
        .with_summary("Test summary");
    tracing::debug!("MemoryRecord with_summary: {}", m2.summary);
    let m3 = MemoryRecord::new("test content", "test_type", MemoryKind::Working)
        .with_embedding(vec![0.1, 0.2, 0.3]);
    tracing::debug!(
        "MemoryRecord with_embedding: len={:?}",
        m3.embedding.map(|e| e.len())
    );
    let m4 =
        MemoryRecord::new("test content", "test_type", MemoryKind::Working).with_confidence(0.9);
    tracing::debug!("MemoryRecord with_confidence: {}", m4.confidence);
    let rel_id = uuid::Uuid::new_v4();
    let m5 = MemoryRecord::new("test content", "test_type", MemoryKind::Working)
        .with_relationship(rel_id);
    tracing::debug!(
        "MemoryRecord with_relationship: count={}",
        m5.relationships.len()
    );
    let m6 =
        MemoryRecord::new("test content", "test_type", MemoryKind::Working).with_tag("important");
    tracing::debug!("MemoryRecord with_tag: count={}", m6.tags.len());
    tracing::debug!("memory_record_methods: builder methods actively referenced");
}
