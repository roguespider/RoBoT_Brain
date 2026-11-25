//! Task Queue — Per Architecture Chapter 23.5.
//!
//! Background task queue for the worker system.

use chrono::Utc;
use serde_json::Value;

/// A queued background task.
#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundTask {
    /// Task identifier.
    pub id: String,
    /// Type of task.
    pub task_type: String,
    /// Priority level.
    pub priority: String,
    /// Status.
    pub status: String,
    /// Payload data.
    pub payload: Value,
    /// Created timestamp.
    pub created_at: i64,
}

impl BackgroundTask {
    /// Create a new background task.
    pub fn new(
        id: impl Into<String>,
        task_type: impl Into<String>,
        priority: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            id: id.into(),
            task_type: task_type.into(),
            priority: priority.into(),
            status: "pending".to_string(),
            payload,
            created_at: Utc::now().timestamp(),
        }
    }
}
