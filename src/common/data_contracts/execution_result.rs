/// ExecutionResult data contract - Per Architecture Chapter 5.10.
///
/// ExecutionResult captures what actually occurred during execution.
/// Per Architecture §5.10: executed skills, tools used, outputs, errors,
/// warnings, execution metrics, resource usage, completion status.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// The result of executing a step or tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResult {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// The step ID that was executed.
    pub step_id: String,
    /// Whether the execution was successful.
    pub success: bool,
    /// The output of the execution.
    pub output: String,
    /// Error message if execution failed.
    pub error: Option<String>,
    /// Skills executed during this step.
    pub executed_skills: Vec<String>,
    /// Tools used during execution.
    pub tools_used: Vec<String>,
    /// Warnings produced during execution.
    pub warnings: Vec<String>,
    /// Execution metrics (e.g., duration, resource usage).
    pub execution_metrics: std::collections::HashMap<String, f64>,
    /// Resource usage details.
    pub resource_usage: String,
    /// Completion status of the execution.
    pub completion_status: String,
    /// Duration of execution in milliseconds.
    pub duration_ms: u64,
}

impl ExecutionResult {
    /// Create a new execution result.
    pub fn new(step_id: impl Into<String>, success: bool, output: impl Into<String>) -> Self {
        Self {
            metadata: Metadata::new("execution_result"),
            step_id: step_id.into(),
            success,
            output: output.into(),
            error: None,
            executed_skills: Vec::new(),
            tools_used: Vec::new(),
            warnings: Vec::new(),
            execution_metrics: std::collections::HashMap::new(),
            resource_usage: String::new(),
            completion_status: if success {
                "completed".to_string()
            } else {
                "failed".to_string()
            },
            duration_ms: 0,
        }
    }

    /// Add an executed skill.
    pub fn with_skill(mut self, skill: impl Into<String>) -> Self {
        self.executed_skills.push(skill.into());
        self
    }

    /// Add a tool used.
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools_used.push(tool.into());
        self
    }

    /// Add a warning.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Set resource usage.
    pub fn with_resource_usage(mut self, usage: impl Into<String>) -> Self {
        self.resource_usage = usage.into();
        self
    }

    /// Set duration.
    pub fn with_duration(mut self, millis: u64) -> Self {
        self.duration_ms = millis;
        self
    }
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            step_id: String::new(),
            success: false,
            output: String::new(),
            error: None,
            executed_skills: Vec::new(),
            tools_used: Vec::new(),
            warnings: Vec::new(),
            execution_metrics: std::collections::HashMap::new(),
            resource_usage: String::new(),
            completion_status: "pending".to_string(),
            duration_ms: 0,
        }
    }
}

