//! Sandbox model for the safety gate (Architecture §16 — sandboxing).
//!
//! The sandbox defines the resource boundary within which the autonomous loop
//! may operate. Actions outside the sandbox boundary are blocked before
//! execution. This is the first line of defense against an autonomous loop
//! taking unintended actions.
//!
//! The sandbox is configured with:
//!   * An **action allow-list** — which tool names the loop may call.
//!   * A **mutation budget** — max mutations per loop iteration to prevent
//!     runaway write amplification.
//!   * A **resource scope** — what subsystems (memory, knowledge, planner)
//!     the loop may touch.

use std::collections::HashSet;

use super::types::ActionRisk;

/// Resource subsystems the sandbox can grant access to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceScope {
    Memory,
    Knowledge,
    Experience,
    Planner,
    Workflow,
    Acp,
    External,
}

/// Configuration for the sandbox.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum mutations permitted per loop iteration. Prevents runaway write
    /// amplification (Architecture §16 sandboxing).
    pub max_mutations_per_iteration: usize,
    /// Resource scopes the loop is permitted to access.
    pub allowed_scopes: HashSet<ResourceScope>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        let mut allowed_scopes = HashSet::new();
        allowed_scopes.insert(ResourceScope::Memory);
        allowed_scopes.insert(ResourceScope::Knowledge);
        allowed_scopes.insert(ResourceScope::Experience);
        allowed_scopes.insert(ResourceScope::Planner);
        allowed_scopes.insert(ResourceScope::Workflow);
        Self {
            max_mutations_per_iteration: 5,
            allowed_scopes,
        }
    }
}

/// The sandbox enforces resource boundaries on autonomous actions.
pub struct Sandbox {
    config: SandboxConfig,
    /// Mutations executed in the current iteration.
    mutations_this_iteration: usize,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            mutations_this_iteration: 0,
        }
    }

    /// Check whether the action falls within the sandbox boundary.
    ///
    /// Returns `Ok(())` if the action is permitted, `Err(reason)` if blocked.
    pub fn check(&mut self, action: &str) -> Result<(), String> {
        let risk = super::action_risk(action);

        // Destructive actions are always blocked by the sandbox.
        if risk == ActionRisk::Destructive {
            return Err(format!(
                "Sandbox: action '{}' is destructive and outside the allow-list",
                action
            ));
        }

        // Enforce the mutation budget.
        if risk == ActionRisk::Mutate {
            if self.mutations_this_iteration >= self.config.max_mutations_per_iteration {
                return Err(format!(
                    "Sandbox: mutation budget exhausted ({} mutations this iteration)",
                    self.config.max_mutations_per_iteration
                ));
            }
            self.mutations_this_iteration += 1;
        }

        // Check resource scope. Map the action to a scope and verify it's
        // in the allowed set.
        let scope = Self::action_scope(action);
        if !self.config.allowed_scopes.contains(&scope) {
            return Err(format!(
                "Sandbox: action '{}' targets scope {:?} which is outside the allowed resource boundary",
                action, scope
            ));
        }

        Ok(())
    }

    /// Reset the per-iteration mutation counter (called at the start of each
    /// new agent loop iteration).
    pub fn reset_iteration(&mut self) {
        self.mutations_this_iteration = 0;
    }

    /// Map an action name to the resource scope it accesses.
    fn action_scope(action: &str) -> ResourceScope {
        match action {
            "search_memory" | "store_memory" | "get_memory" | "list_memories"
            | "archive_memory" | "link_memories" | "ranked_search" => ResourceScope::Memory,
            "query_knowledge" | "add_knowledge" | "get_knowledge" | "global_search" => {
                ResourceScope::Knowledge
            }
            "record_experience" | "list_experiences" | "get_insights" => {
                ResourceScope::Experience
            }
            "create_plan" | "get_plan" | "list_plans" => ResourceScope::Planner,
            "create_workflow" | "start_workflow" | "list_workflows" => ResourceScope::Workflow,
            "register_agent" | "route_acp_message" | "list_agents" => ResourceScope::Acp,
            _ => ResourceScope::External,
        }
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new(SandboxConfig::default())
    }
}
