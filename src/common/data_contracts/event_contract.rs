/// Event contract — Per Architecture Chapter 5.24 (Event Contracts) and Appendix C (Event Definitions).
///
/// Defines the canonical event contract used to report something that happened
/// across the cognitive architecture.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A cognitive event reporting something that occurred in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    /// Shared metadata.
    pub metadata: Metadata,
    /// Event type/category (e.g., "memory_created", "experience_completed").
    pub event_type: String,
    /// Human-readable description of the event.
    pub description: String,
    /// Related entity IDs (e.g., memory IDs, goal IDs).
    pub related_ids: Vec<String>,
    /// Event payload (structured data).
    pub payload: serde_json::Value,
    /// Severity level.
    pub severity: String,
}

impl Event {
    /// Create a new event.
    pub fn new(event_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("event_contract"),
            event_type: event_type.into(),
            description: description.into(),
            related_ids: Vec::new(),
            payload: serde_json::Value::Object(serde_json::Map::new()),
            severity: "info".to_string(),
        }
    }

    /// Add related entity IDs.
    pub fn with_related_ids(mut self, ids: Vec<String>) -> Self {
        self.related_ids = ids;
        self
    }

    /// Set the payload.
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Set the severity.
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = severity.into();
        self
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            event_type: String::new(),
            description: String::new(),
            related_ids: Vec::new(),
            payload: serde_json::Value::Object(serde_json::Map::new()),
            severity: "info".to_string(),
        }
    }
}

