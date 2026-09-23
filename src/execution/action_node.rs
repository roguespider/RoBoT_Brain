//! Execution Action Node — Individual executable action (Architecture Chapter 12.8).
//!
//! Wiring: `execution/` -> `tools/` -> `execution/`

/// An executable action node connecting to tools.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActionNode {
    /// Node identifier.
    pub id: String,
    /// Tool or module to invoke.
    pub tool: String,
    /// Parameters for the tool.
    pub parameters: std::collections::HashMap<String, String>,
    /// Whether execution is complete.
    pub completed: bool,
    /// Result from execution.
    pub result: Option<String>,
}

impl ActionNode {
    /// Create a new action node.
    pub fn new(id: &str, tool: &str) -> Self {
        Self {
            id: id.to_string(),
            tool: tool.to_string(),
            parameters: std::collections::HashMap::new(),
            completed: false,
            result: None,
        }
    }

    /// Add a parameter.
    pub fn with_parameter(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    /// Mark as completed with result.
    pub fn complete(&mut self, result: &str) {
        self.completed = true;
        self.result = Some(result.to_string());
    }

    /// Create a default/unknown node (used as fallback).
    pub fn default_node() -> Self {
        Self::new("unknown", "unknown")
    }
}
