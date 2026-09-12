use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::metadata::Metadata;
use crate::memory::lifecycle::MemoryLifecycle;
use crate::memory::types::MemoryLayer;

/// MemoryRecord data contract - Per Architecture §5.1 + §8.1
///
/// Canonical memory record shared across all subsystems.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRecord {
    /// Unique identifier
    pub id: Uuid,
    /// Memory content
    pub content: String,
    /// Memory layer (Working or Permanent)
    pub layer: MemoryLayer,
    /// Memory lifecycle state
    pub lifecycle: MemoryLifecycle,
    /// Memory type classification
    pub memory_type: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// When this memory was created (Unix timestamp)
    pub created_at: i64,
    /// When this memory was last accessed (Unix timestamp)
    pub accessed_at: i64,
    /// When this memory was last modified (Unix timestamp)
    pub modified_at: i64,
    /// Access count
    pub access_count: u32,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Source of this memory (experience, user, system)
    pub source: String,
    /// Source kind (user_input, research, experience, system)
    pub source_kind: String,
    /// Related memory IDs
    pub related_ids: Vec<Uuid>,
    /// Metadata shared across data contracts
    pub metadata: Metadata,
    /// Whether this memory is an anchor (permanent by definition)
    pub is_anchor: bool,
    /// Whether this memory has been consolidated into another
    pub consolidated_from: Vec<String>,
    /// Whether this memory was summarized into another
    pub summarized_into: Option<String>,
    /// Provenance records
    pub provenance: Vec<String>,
}

impl MemoryRecord {
    /// Create a new memory record
    pub fn new(content: String, memory_type: String, source: String, source_kind: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4(),
            content,
            layer: MemoryLayer::Working,
            lifecycle: MemoryLifecycle::Working,
            memory_type,
            confidence: 0.5,
            importance: 0.5,
            created_at: now,
            accessed_at: now,
            modified_at: now,
            access_count: 0,
            tags: Vec::new(),
            source,
            source_kind,
            related_ids: Vec::new(),
            metadata: Metadata::new("memory"),
            is_anchor: false,
            consolidated_from: Vec::new(),
            summarized_into: None,
            provenance: Vec::new(),
        }
    }

    /// Record an access to this memory
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.accessed_at = Utc::now().timestamp();
        self.modified_at = self.accessed_at;
    }

    /// Archive this memory
    pub fn archive(&mut self) {
        self.lifecycle = MemoryLifecycle::Archived;
        self.modified_at = Utc::now().timestamp();
    }

    /// Promote this memory to the next lifecycle state
    pub fn promote(&mut self) -> bool {
        let next = match self.lifecycle {
            MemoryLifecycle::Working => Some(MemoryLifecycle::Candidate),
            MemoryLifecycle::Candidate => Some(MemoryLifecycle::Accepted),
            MemoryLifecycle::Accepted => Some(MemoryLifecycle::Permanent),
            MemoryLifecycle::Permanent => None,
            MemoryLifecycle::Archived => None,
        };

        match next {
            Some(state) => {
                self.lifecycle = state;
                self.modified_at = Utc::now().timestamp();
                true
            }
            None => false,
        }
    }
}

impl Default for MemoryRecord {
    fn default() -> Self {
        Self::new(
            String::new(),
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
        )
    }
}
