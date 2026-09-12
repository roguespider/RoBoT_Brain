/// Common metadata fields for all data contract types.
///
/// Provides version, source identification, and timestamp tracking
/// across all subsystems.
use serde::{Deserialize, Serialize};

use super::version::{CONTRACT_VERSION, Versioned};
use chrono;
use uuid;

/// Metadata shared by all data contracts.
///
/// Includes version for serialization compatibility, source identification
/// for provenance tracking, and created_at timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    /// Contract version this record conforms to.
    pub version: String,
    /// Origin system or component that created this record.
    pub source: String,
    /// Unix timestamp (seconds) when the record was created.
    pub created_at: i64,
    /// Correlation ID for tracing related operations across subsystems.
    pub correlation_id: String,
    /// Confidence score (0.0-1.0) for this record's reliability.
    pub confidence: f32,
    /// List of source IDs that contributed to this record's creation.
    pub provenance: Vec<String>,
}

impl Metadata {
    /// Create a new metadata instance with sensible defaults.
    pub fn new(source: &str) -> Self {
        Self {
            version: CONTRACT_VERSION.to_string(),
            source: source.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            confidence: 0.5,
            provenance: Vec::new(),
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new("unknown")
    }
}

impl Versioned for Metadata {
    fn version() -> &'static str {
        CONTRACT_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_defaults() {
        let meta = Metadata::default();
        assert_eq!(meta.version, CONTRACT_VERSION);
        assert_eq!(meta.source, "unknown");
        assert!(meta.created_at > 0);
        assert!(!meta.correlation_id.is_empty());
        assert_eq!(meta.confidence, 0.5);
        assert!(meta.provenance.is_empty());
    }

    #[test]
    fn test_metadata_source() {
        let meta = Metadata::new("test_source");
        assert_eq!(meta.source, "test_source");
    }
}
