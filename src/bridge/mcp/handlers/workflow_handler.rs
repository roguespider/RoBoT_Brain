// src/bridge/tools/handlers/workflow_handler.rs
// Workflow tools handler - handles workflow creation and execution

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::workflow;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};

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

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "create_workflow",
                "Create a new workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Workflow name" }
                    },
                    "required": ["name"]
                })),
            ).with_title("Create Workflow"),
            rmcp::model::Tool::new(
                "add_workflow_step",
                "Add a step to a workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" },
                        "name": { "type": "string", "description": "Step name" },
                        "action": { "type": "string", "description": "Tool/action to execute" },
                        "parameters": { "type": "string", "description": "JSON parameters for the action" }
                    },
                    "required": ["workflow_id", "name", "action"]
                })),
            ).with_title("Add Workflow Step"),
            rmcp::model::Tool::new(
                "get_workflow_status",
                "Get the status of a workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" }
                    },
                    "required": ["workflow_id"]
                })),
            ).with_title("Get Workflow Status"),
            rmcp::model::Tool::new(
                "list_workflows",
                "List all workflows",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("List Workflows"),
            rmcp::model::Tool::new(
                "start_workflow",
                "Start a workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" }
                    },
                    "required": ["workflow_id"]
                })),
            ).with_title("Start Workflow"),
            rmcp::model::Tool::new(
                "pause_workflow",
                "Pause a running workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" }
                    },
                    "required": ["workflow_id"]
                })),
            ).with_title("Pause Workflow"),
            rmcp::model::Tool::new(
                "resume_workflow",
                "Resume a paused workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" }
                    },
                    "required": ["workflow_id"]
                })),
            ).with_title("Resume Workflow"),
            rmcp::model::Tool::new(
                "cancel_workflow",
                "Cancel a workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" }
                    },
                    "required": ["workflow_id"]
                })),
            ).with_title("Cancel Workflow"),
            rmcp::model::Tool::new(
                "delete_workflow",
                "Delete a workflow",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "workflow_id": { "type": "string", "description": "Workflow ID" }
                    },
                    "required": ["workflow_id"]
                })),
            ).with_title("Delete Workflow"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        use crate::bridge::tools::workflow;
        async move {
            match name {
                "create_workflow" => {
                    let input: workflow::CreateWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_create_workflow(input).await)
                }
                "add_workflow_step" => {
                    let input: workflow::AddWorkflowStepInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_add_workflow_step(input).await)
                }
                "get_workflow_status" => {
                    let input: workflow::GetWorkflowStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_get_workflow_status(input).await)
                }
                "list_workflows" => {
                    let input: workflow::ListWorkflowsInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_list_workflows(input).await)
                }
                "start_workflow" => {
                    let input: workflow::StartWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_start_workflow(input).await)
                }
                "pause_workflow" => {
                    let input: workflow::PauseWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_pause_workflow(input).await)
                }
                "resume_workflow" => {
                    let input: workflow::ResumeWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_resume_workflow(input).await)
                }
                "cancel_workflow" => {
                    let input: workflow::CancelWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_cancel_workflow(input).await)
                }
                "delete_workflow" => {
                    let input: workflow::DeleteWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_delete_workflow(input).await)
                }
                other => Err(HandlerError::ToolNotFound(other.to_string())),
            }
        }
    }
}
