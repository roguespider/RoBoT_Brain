    // workflow_tools.rs - Workflow management tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::bridge::tools;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

#[tool_router]
impl McpServerHandler {
    #[tool(name = "create_workflow", description = "Create a new workflow with a name and optional description")]
    async fn create_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::CreateWorkflowInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("create_workflow").await {
            tracing::warn!("Workflow enforcement blocked create_workflow: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_create_workflow(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("create_workflow", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "add_workflow_step", description = "Add a step to an existing workflow.")]
    async fn add_workflow_step(
        &self,
        Parameters(input): Parameters<tools::workflow::AddWorkflowStepInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("add_workflow_step").await {
            tracing::warn!("Workflow enforcement blocked add_workflow_step: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_add_workflow_step(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("add_workflow_step", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "get_workflow_status", description = "Get the current status and details of a workflow")]
    async fn get_workflow_status(
        &self,
        Parameters(input): Parameters<tools::workflow::GetWorkflowStatusInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_workflow_status").await {
            tracing::warn!("Workflow enforcement blocked get_workflow_status: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_get_workflow_status(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("get_workflow_status", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "list_workflows", description = "List all workflows, optionally filtered by status")]
    async fn list_workflows(
        &self,
        Parameters(input): Parameters<tools::workflow::ListWorkflowsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("list_workflows").await {
            tracing::warn!("Workflow enforcement blocked list_workflows: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_list_workflows(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("list_workflows", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "start_workflow", description = "Start executing a workflow.")]
    async fn start_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::StartWorkflowInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("start_workflow").await {
            tracing::warn!("Workflow enforcement blocked start_workflow: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_start_workflow(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("start_workflow", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "pause_workflow", description = "Pause a running workflow")]
    async fn pause_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::PauseWorkflowInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("pause_workflow").await {
            tracing::warn!("Workflow enforcement blocked pause_workflow: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_pause_workflow(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("pause_workflow", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "resume_workflow", description = "Resume a paused workflow")]
    async fn resume_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::ResumeWorkflowInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("resume_workflow").await {
            tracing::warn!("Workflow enforcement blocked resume_workflow: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_resume_workflow(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("resume_workflow", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "cancel_workflow", description = "Cancel a workflow, removing it from execution.")]
    async fn cancel_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::CancelWorkflowInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("cancel_workflow").await {
            tracing::warn!("Workflow enforcement blocked cancel_workflow: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_cancel_workflow(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("cancel_workflow", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(name = "delete_workflow", description = "Delete a workflow completely.")]
    async fn delete_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::DeleteWorkflowInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("delete_workflow").await {
            tracing::warn!("Workflow enforcement blocked delete_workflow: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let result = tools::workflow::execute_delete_workflow(input, &self.context.workflow_engine).await;
        if result.success {
            self.record_tool_execution("delete_workflow", None).await;
        }
        tool_output_to_content(result)
    }
}
