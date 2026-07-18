use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single thing tried during exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationAttempt {
    pub id: String,

    pub timestamp: DateTime<Utc>,

    /// What was attempted
    pub action: String,

    /// Expected outcome
    pub expected_result: Option<String>,

    /// Actual outcome
    pub actual_result: Option<String>,

    /// Did this attempt succeed?
    pub success: bool,
}
