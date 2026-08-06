// workflow_tools.rs - Workflow execution tools

use crate::bridge::rmcp::generated::tool_traits::{
    WorkflowToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for workflow tools - implements WorkflowToolsHandlerTrait
pub struct WorkflowToolsHandler;

impl WorkflowToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WorkflowToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowToolsHandlerTrait for WorkflowToolsHandler {
    async fn execute_create_workflow(
        &self,
        context: &ToolContext,
        input: tools::workflow::CreateWorkflowInput,
    ) -> ToolOutput {
        match tools::workflow::execute_create_workflow(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_workflow(
        &self,
        context: &ToolContext,
        input: tools::workflow::GetWorkflowInput,
    ) -> ToolOutput {
        match tools::workflow::execute_get_workflow(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_workflows(
        &self,
        context: &ToolContext,
        input: tools::workflow::ListWorkflowsInput,
    ) -> ToolOutput {
        match tools::workflow::execute_list_workflows(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_execute_workflow(
        &self,
        context: &ToolContext,
        input: tools::workflow::ExecuteWorkflowInput,
    ) -> ToolOutput {
        match tools::workflow::execute_execute_workflow(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_pause_workflow(
        &self,
        context: &ToolContext,
        input: tools::workflow::PauseWorkflowInput,
    ) -> ToolOutput {
        match tools::workflow::execute_pause_workflow(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_resume_workflow(
        &self,
        context: &ToolContext,
        input: tools::workflow::ResumeWorkflowInput,
    ) -> ToolOutput {
        match tools::workflow::execute_resume_workflow(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_cancel_workflow(
        &self,
        context: &ToolContext,
        input: tools::workflow::CancelWorkflowInput,
    ) -> ToolOutput {
        match tools::workflow::execute_cancel_workflow(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_workflow_status(
        &self,
        context: &ToolContext,
        input: tools::workflow::GetWorkflowStatusInput,
    ) -> ToolOutput {
        match tools::workflow::execute_get_workflow_status(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_workflow_executions(
        &self,
        context: &ToolContext,
        input: tools::workflow::ListWorkflowExecutionsInput,
    ) -> ToolOutput {
        match tools::workflow::execute_list_workflow_executions(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_workflow_execution(
        &self,
        context: &ToolContext,
        input: tools::workflow::GetWorkflowExecutionInput,
    ) -> ToolOutput {
        match tools::workflow::execute_get_workflow_execution(input, &context.context.database)
            .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::workflow::create_workflow_tool(),
            tools::workflow::get_workflow_tool(),
            tools::workflow::list_workflows_tool(),
            tools::workflow::execute_workflow_tool(),
            tools::workflow::pause_workflow_tool(),
            tools::workflow::resume_workflow_tool(),
            tools::workflow::cancel_workflow_tool(),
            tools::workflow::get_workflow_status_tool(),
            tools::workflow::list_workflow_executions_tool(),
            tools::workflow::get_workflow_execution_tool(),
        ]
    }
}
