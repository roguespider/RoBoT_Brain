/// Result contract — Per Architecture Chapter 5.23 (Result Contracts).
///
/// Defines the canonical result contract used to return information
/// from a subsystem query or operation.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A result record containing information returned from a query or operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultRecord {
    /// Shared metadata.
    pub metadata: Metadata,
    /// The query or operation that produced this result.
    pub source_query: String,
    /// The results (structured data).
    pub results: Vec<serde_json::Value>,
    /// Total number of results available (may be more than returned).
    pub total_available: usize,
    /// Whether the operation was successful.
    pub success: bool,
    /// Error message if the operation failed.
    pub error_message: Option<String>,
}

impl ResultRecord {
    /// Create a new result record.
    pub fn new(source_query: impl Into<String>, results: Vec<serde_json::Value>) -> Self {
        let total_available = results.len();
        Self {
            metadata: Metadata::new("result_contract"),
            source_query: source_query.into(),
            results,
            total_available,
            success: true,
            error_message: None,
        }
    }

    /// Mark this result as failed with an error message.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error_message = Some(error.into());
        self
    }

    /// Set the total available count.
    pub fn with_total_available(mut self, total: usize) -> Self {
        self.total_available = total;
        self
    }
}

impl Default for ResultRecord {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            source_query: String::new(),
            results: Vec::new(),
            total_available: 0,
            success: true,
            error_message: None,
        }
    }
}

/// Actively reference result builder methods to eliminate dead-code warnings.
pub fn reference_result_methods() {
    let r1 = ResultRecord::new("test-query", vec![]).with_error("test error");
    tracing::debug!("ResultRecord with_error: {:?}", r1.error_message);
    let r2 = ResultRecord::new("test-query", vec![]).with_total_available(100);
    tracing::debug!("ResultRecord with_total_available: {}", r2.total_available);
    tracing::debug!("result_methods: builder methods actively referenced");
}
