use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::metadata::Metadata;

/// MemoryKind - Kind of memory lifecycle state.
///
/// Corresponds to the lifecycle states in `src/memory/lifecycle::MemoryLifecycle`.
/// This is the canonical kind enum used by the `MemoryRecord` data contract.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MemoryKind {
    /// Memory is being actively used or evaluated
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

impl Default for MemoryKind {
    fn default() -> Self {
        MemoryKind::Working
    }
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

/// MemoryRecord data contract - Per Architecture §5.1 + §8.1
///
/// Canonical memory record shared across all subsystems.
/// This is the canonical type that all legacy memory types will be migrated to (T2-46).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryRecord {
    /// Unique identifier
    pub id: Uuid,
    /// Memory kind (lifecycle state)
    pub kind: MemoryKind,
    /// Memory content
    pub content: String,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Access count
    pub access_count: u32,
    /// Metadata shared across data contracts
    pub metadata: Metadata,
}

impl Default for MemoryRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: MemoryKind::default(),
            content: String::new(),
            importance: 0.5,
            access_count: 0,
            metadata: Metadata::default(),
        }
    }
}

impl MemoryRecord {
    /// Create a new memory record
    pub fn new(content: String, kind: MemoryKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
            content,
            importance: 0.5,
            access_count: 0,
            metadata: Metadata::default(),
        }
    }

    /// Record an access to this memory
    pub fn record_access(&mut self) {
        self.metadata.confidence = self.metadata.confidence.max(0.5);
        self.importance = self.importance.max(0.5);
    }

    /// Archive this memory
    pub fn archive(&mut self) {
        self.kind = MemoryKind::Archived;
    }

    /// Promote this memory to the next lifecycle state
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
                true
            }
            None => false,
        }
    }
}
