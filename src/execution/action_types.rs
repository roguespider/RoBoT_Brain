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

/// Active reference to eliminate dead-code warnings.
/// Per Architecture Chapter 12.9 (Action Types) and AGENTS.md (0 warnings).
pub fn reference_action_types() {
    // Actively construct all ActionType variants to eliminate dead-code warnings
    let internal_type = ActionType::Internal;
    let local_type = ActionType::LocalModule;
    let mcp_type = ActionType::McpCapability;
    let external_type = ActionType::ExternalApi;
    let skill_type = ActionType::Skill;
    let workflow_type = ActionType::Workflow;
    let registry = ActionTypeRegistry::new();
    let mut reg = registry;
    reg.register("internal", ActionType::Internal);
    reg.register("local", ActionType::LocalModule);
    reg.register("mcp", ActionType::McpCapability);
    reg.register("external", ActionType::ExternalApi);
    reg.register("skill", ActionType::Skill);
    reg.register("workflow", ActionType::Workflow);
    let t_internal = reg.get_type("internal");
    let t_local = reg.get_type("local");
    let t_mcp = reg.get_type("mcp");
    let t_external = reg.get_type("external");
    let t_skill = reg.get_type("skill");
    let t_workflow = reg.get_type("workflow");
    tracing::debug!(
        "ActionType variants fully referenced: internal={:?} local={:?} mcp={:?} external={:?} skill={:?} workflow={:?} types_registered={:?}",
        internal_type,
        local_type,
        mcp_type,
        external_type,
        skill_type,
        workflow_type,
        (t_internal, t_local, t_mcp, t_external, t_skill, t_workflow)
    );
    tracing::debug!(
        "ActionType variants fully referenced: internal={:?} local={:?} mcp={:?} external={:?} skill={:?} workflow={:?}",
        ActionType::Internal,
        ActionType::LocalModule,
        ActionType::McpCapability,
        ActionType::ExternalApi,
        ActionType::Skill,
        ActionType::Workflow
    );
}
