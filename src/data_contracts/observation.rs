/// Observation data contract (Chapter 5.1 + Chapter 4.2 "Input processing").
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A single observation from the environment or user input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// Kind of source that produced this observation (e.g. "user_input", "system_event").
    pub source_kind: String,
    /// The observed content.
    pub content: String,
    /// Tags for categorization and retrieval.
    pub tags: Vec<String>,
}

impl Observation {
    /// Create a new observation with the given source kind and content.
    pub fn new(source_kind: &str, content: &str) -> Self {
        Self {
            metadata: Metadata::new(source_kind),
            source_kind: source_kind.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
        }
    }
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            source_kind: "unknown".to_string(),
            content: String::new(),
            tags: Vec::new(),
        }
    }
}
