//! ExplorationAttempt - a single attempt made during exploration.
//!
//! Per Architecture §2.7, attempts record what was tried during exploration,
//! including expected and actual outcomes for learning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single attempt made during exploration.
///
/// Records what was tried, what was expected, and what actually happened.
/// This enables learning from both successes and failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationAttempt {
    /// Unique attempt identifier
    pub id: String,

    /// When the attempt was made
    pub timestamp: DateTime<Utc>,

    /// What action was taken
    pub action: String,

    /// What outcome was expected
    pub expected_result: Option<String>,

    /// What outcome actually occurred
    pub actual_result: Option<String>,

    /// Whether the attempt succeeded
    pub success: bool,
}

impl ExplorationAttempt {
    /// Create a new attempt with the given action.
    #[allow(dead_code)]
    pub fn new(id: String, action: String) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            action,
            expected_result: None,
            actual_result: None,
            success: false,
        }
    }

    /// Set the expected result before executing.
    #[allow(dead_code)]
    pub fn with_expected_result(mut self, result: String) -> Self {
        self.expected_result = Some(result);
        self
    }

    /// Record the actual result and determine success.
    #[allow(dead_code)]
    pub fn with_actual_result(mut self, result: String) -> Self {
        let result_clone = result.clone();
        self.actual_result = Some(result);
        self.success = self.expected_result.as_ref()
            .map(|e| e == &result_clone)
            .unwrap_or(false);
        self
    }
}
