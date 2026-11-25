//! Execution Action Types — Classification of executable actions (Architecture Chapter 12.9).
//!
//! Wiring: `execution/` -> `skills/` + `workflows/` + `bridge/mcp/`

/// Types of executable actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionType {
    /// Internal operation (local Rust module).
    Internal,
    /// Local module invocation.
    LocalModule,
    /// MCP capability call.
    McpCapability,
    /// External API call.
    ExternalApi,
    /// Skill execution.
    Skill,
    /// Workflow step.
    Workflow,
}

/// Action type registry mapping actions to their types.
#[derive(Debug, Clone, Default)]
pub struct ActionTypeRegistry {
    /// Action to type mappings.
    mappings: std::collections::HashMap<String, ActionType>,
}

impl ActionTypeRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            mappings: std::collections::HashMap::new(),
        }
    }

    /// Register an action type.
    pub fn register(&mut self, action: &str, action_type: ActionType) {
        self.mappings.insert(action.to_string(), action_type);
    }

    /// Get the type for an action.
    pub fn get_type(&self, action: &str) -> Option<ActionType> {
        self.mappings.get(action).cloned()
    }
}

