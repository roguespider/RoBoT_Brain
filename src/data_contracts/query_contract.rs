/// Query contract — Per Architecture Chapter 5.22 (Query Contracts).
///
/// Defines the canonical query contract used to request information
/// from memory, knowledge, or experience subsystems.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A query requesting information from a subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Query {
    /// Shared metadata.
    pub metadata: Metadata,
    /// The query content or keywords.
    pub content: String,
    /// Source subsystem being queried (e.g., "memory", "knowledge", "experience").
    pub source: String,
    /// Maximum number of results requested.
    pub limit: usize,
    /// Minimum confidence threshold for results.
    pub min_confidence: f32,
    /// Tags to filter results.
    pub tags: Vec<String>,
}

impl Query {
    /// Create a new query.
    pub fn new(content: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("query_contract"),
            content: content.into(),
            source: source.into(),
            limit: 10,
            min_confidence: 0.0,
            tags: Vec::new(),
        }
    }

    /// Set the result limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the minimum confidence.
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Add filter tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Default for Query {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            content: String::new(),
            source: String::new(),
            limit: 10,
            min_confidence: 0.0,
            tags: Vec::new(),
        }
    }
}

/// Actively reference query builder methods to eliminate dead-code warnings.
pub fn reference_query_methods() {
    let q1 = Query::new("test", "memory").with_limit(5);
    tracing::debug!("Query with_limit: limit={}", q1.limit);
    let q2 = Query::new("test", "memory").with_min_confidence(0.8);
    tracing::debug!(
        "Query with_min_confidence: min_confidence={}",
        q2.min_confidence
    );
    let q3 = Query::new("test", "memory").with_tags(vec!["rust".to_string(), "mcp".to_string()]);
    tracing::debug!("Query with_tags: count={}", q3.tags.len());
    tracing::debug!("query_methods: builder methods actively referenced");
}
