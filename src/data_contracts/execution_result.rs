/// ExecutionResult data contract - Per Architecture Chapter 12.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// The result of executing a step or tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Shared metadata (version, source, timestamp, correlation, confidence).
    pub metadata: Metadata,
    /// The step ID that was executed.
    pub step_id: String,
    /// Whether the execution was successful.
    pub success: bool,
    /// The output of the execution.
    pub output: String,
    /// Error message if execution failed.
    pub error: Option<String>,
    /// Duration of execution in milliseconds.
    pub duration_ms: u64,
}

impl ExecutionResult {
    pub fn new(step_id: impl Into<String>, success: bool, output: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("execution_result"),
            step_id: step_id.into(),
            success,
            output: output.into(),
            error: None,
            duration_ms: 0,
        }
    }
}
