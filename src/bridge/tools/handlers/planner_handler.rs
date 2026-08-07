// src/bridge/tools/handlers/planner_handler.rs
// Planner tools handler - handles task planning and execution

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::planner;
use crate::bridge::tools::handlers::{HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for planner-related tools
#[derive(Clone)]
pub struct PlannerToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl PlannerToolsHandler {
    /// Create a new planner tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Planner is available - async validation happens at runtime
        Ok(Self { context, enforcer })
    }

    /// Create a new plan
    pub async fn execute_create_plan(
        &self,
        input: planner::CreatePlanInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_create_plan(input, &self.context.planner).await
    }

    /// Add a step to a plan
    pub async fn execute_add_plan_step(
        &self,
        input: planner::AddPlanStepInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_add_plan_step(input, &self.context.planner).await
    }

    /// Add a dependency between steps
    pub async fn execute_add_step_dependency(
        &self,
        input: planner::AddStepDependencyInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_add_step_dependency(input, &self.context.planner).await
    }

    /// Get a plan by ID
    pub async fn execute_get_plan(
        &self,
        input: planner::GetPlanInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_get_plan(input, &self.context.planner).await
    }

    /// List all plans
    pub async fn execute_list_plans(
        &self,
        input: planner::ListPlansInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_list_plans(input, &self.context.planner).await
    }

    /// Start executing a plan
    pub async fn execute_start_plan(
        &self,
        input: planner::StartPlanInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_start_plan(input, &self.context.planner).await
    }

    /// Complete a step
    pub async fn execute_complete_step(
        &self,
        input: planner::CompleteStepInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_complete_step(input, &self.context.planner).await
    }

    /// Fail a step
    pub async fn execute_fail_step(
        &self,
        input: planner::FailStepInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_fail_step(input, &self.context.planner).await
    }

    /// Cancel a plan
    pub async fn execute_cancel_plan(
        &self,
        input: planner::CancelPlanInput,
    ) -> crate::bridge::tools::ToolOutput {
        planner::execute_cancel_plan(input, &self.context.planner).await
    }
}

impl ToolHandler for PlannerToolsHandler {
    fn category(&self) -> &str {
        "planner"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "create_plan".to_string(),
            "add_plan_step".to_string(),
            "add_step_dependency".to_string(),
            "get_plan".to_string(),
            "list_plans".to_string(),
            "start_plan".to_string(),
            "complete_step".to_string(),
            "fail_step".to_string(),
            "cancel_plan".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        // Planner is considered healthy if we can list plans
        true
    }
}
