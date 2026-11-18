/// Observation data contract (Chapter 5.4 + Chapter 4.2 "Input processing").
///
/// The Observation object represents newly acquired information entering the architecture.
/// Per Architecture §5.4: id, timestamp, source, source_type, content, attachments,
/// metadata, priority, confidence, security_level.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A single observation from the environment or user input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    /// Unique identifier for this observation.
    pub id: String,
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// Timestamp of the observation.
    pub timestamp: i64,
    /// Source that produced this observation.
    pub source: String,
    /// Kind of source (e.g., "user_input", "system_event", "tool_output").
    pub source_type: String,
    /// The observed content.
    pub content: String,
    /// File or data attachments.
    pub attachments: Vec<String>,
    /// Priority of this observation (0.0 - 1.0).
    pub priority: f32,
    /// Confidence in this observation (0.0 - 1.0).
    pub confidence: f32,
    /// Security level of this observation.
    pub security_level: String,
    /// Tags for categorization and retrieval.
    pub tags: Vec<String>,
}

impl Observation {
    /// Create a new observation.
    pub fn new(
        source: impl Into<String>,
        source_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let source_str = source.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: Metadata::new(&source_str),
            timestamp: now,
            source: source_str,
            source_type: source_type.into(),
            content: content.into(),
            attachments: Vec::new(),
            priority: 0.5,
            confidence: 0.5,
            security_level: "standard".to_string(),
            tags: Vec::new(),
        }
    }

    /// Add an attachment.
    pub fn with_attachment(mut self, attachment: impl Into<String>) -> Self {
        self.attachments.push(attachment.into());
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority.clamp(0.0, 1.0);
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set security level.
    pub fn with_security_level(mut self, level: impl Into<String>) -> Self {
        self.security_level = level.into();
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

impl Default for Observation {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: Metadata::default(),
            timestamp: 0,
            source: "unknown".to_string(),
            source_type: "unknown".to_string(),
            content: String::new(),
            attachments: Vec::new(),
            priority: 0.5,
            confidence: 0.5,
            security_level: "standard".to_string(),
            tags: Vec::new(),
        }
    }
}

