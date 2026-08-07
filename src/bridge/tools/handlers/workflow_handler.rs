// src/bridge/tools/handlers/workflow_handler.rs
// Workflow tools handler - handles workflow creation and execution

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::workflow;
use crate::bridge::tools::handlers::{HandlerInitError, HandlerInitResult, ToolHandler};

/// Handler for workflow-related tools
#[derive(Clone)]
pub struct WorkflowToolsHandler {
    context: Arc<McpContext>,
}

impl WorkflowToolsHandler {
    /// Create a new workflow tools handler
    pub fn new(context: Arc<McpContext>) -> HandlerInitResult<Self> {
        Ok(Self { context })
    }

    /// Create a new workflow
    pub async fn execute_create_workflow(
        &self,
        input: workflow::CreateWorkflowInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_create_workflow(input, &self.context.workflow_engine).await
    }

    /// Add a step to a workflow
    pub async fn execute_add_workflow_step(
        &self,
        input: workflow::AddWorkflowStepInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_add_workflow_step(input, &self.context.workflow_engine).await
    }

    /// Get workflow status
    pub async fn execute_get_workflow_status(
        &self,
        input: workflow::GetWorkflowStatusInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_get_workflow_status(input, &self.context.workflow_engine).await
    }

    /// List all workflows
    pub async fn execute_list_workflows(
        &self,
        input: workflow::ListWorkflowsInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_list_workflows(input, &self.context.workflow_engine).await
    }

    /// Start a workflow
    pub async fn execute_start_workflow(
        &self,
        input: workflow::StartWorkflowInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_start_workflow(input, &self.context.workflow_engine).await
    }

    /// Pause a workflow
    pub async fn execute_pause_workflow(
        &self,
        input: workflow::PauseWorkflowInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_pause_workflow(input, &self.context.workflow_engine).await
    }

    /// Resume a paused workflow
    pub async fn execute_resume_workflow(
        &self,
        input: workflow::ResumeWorkflowInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_resume_workflow(input, &self.context.workflow_engine).await
    }

    /// Cancel a workflow
    pub async fn execute_cancel_workflow(
        &self,
        input: workflow::CancelWorkflowInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_cancel_workflow(input, &self.context.workflow_engine).await
    }

    /// Delete a workflow
    pub async fn execute_delete_workflow(
        &self,
        input: workflow::DeleteWorkflowInput,
    ) -> crate::bridge::tools::ToolOutput {
        workflow::execute_delete_workflow(input, &self.context.workflow_engine).await
    }
}

impl ToolHandler for WorkflowToolsHandler {
    fn category(&self) -> &str {
        "workflow"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "create_workflow".to_string(),
            "add_workflow_step".to_string(),
            "get_workflow_status".to_string(),
            "list_workflows".to_string(),
            "start_workflow".to_string(),
            "pause_workflow".to_string(),
            "resume_workflow".to_string(),
            "cancel_workflow".to_string(),
            "delete_workflow".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }
}
