// planner_tools.rs - Planning tools

use crate::bridge::rmcp::generated::tool_traits::{
    PlannerToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for planner tools - implements PlannerToolsHandlerTrait
pub struct PlannerToolsHandler;

impl PlannerToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlannerToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PlannerToolsHandlerTrait for PlannerToolsHandler {
    async fn execute_create_plan(
        &self,
        context: &ToolContext,
        input: tools::planner::CreatePlanInput,
    ) -> ToolOutput {
        match tools::planner::execute_create_plan(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_plan(
        &self,
        context: &ToolContext,
        input: tools::planner::GetPlanInput,
    ) -> ToolOutput {
        match tools::planner::execute_get_plan(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_plans(
        &self,
        context: &ToolContext,
        input: tools::planner::ListPlansInput,
    ) -> ToolOutput {
        match tools::planner::execute_list_plans(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_update_plan(
        &self,
        context: &ToolContext,
        input: tools::planner::UpdatePlanInput,
    ) -> ToolOutput {
        match tools::planner::execute_update_plan(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_plan(
        &self,
        context: &ToolContext,
        input: tools::planner::DeletePlanInput,
    ) -> ToolOutput {
        match tools::planner::execute_delete_plan(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_execute_plan(
        &self,
        context: &ToolContext,
        input: tools::planner::ExecutePlanInput,
    ) -> ToolOutput {
        match tools::planner::execute_execute_plan(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_plan_status(
        &self,
        context: &ToolContext,
        input: tools::planner::GetPlanStatusInput,
    ) -> ToolOutput {
        match tools::planner::execute_get_plan_status(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::planner::create_plan_tool(),
            tools::planner::get_plan_tool(),
            tools::planner::list_plans_tool(),
            tools::planner::update_plan_tool(),
            tools::planner::delete_plan_tool(),
            tools::planner::execute_plan_tool(),
            tools::planner::get_plan_status_tool(),
        ]
    }
}
