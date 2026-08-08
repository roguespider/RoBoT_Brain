// src/bridge/tools/handlers/agent_handler.rs
// Agent tools handler - handles MCP connection and workflow tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::agent::{inputs::*, mcp_tools::*, workflows::*};
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitError, HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for agent and MCP-related tools
#[derive(Clone)]
pub struct AgentToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl AgentToolsHandler {
    /// Create a new agent tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "agent",
                "Database connection not available",
            ));
        }

        Ok(Self { context, enforcer })
    }

    /// Get workflow - MUST be called before any other tool
    pub async fn execute_get_workflow(&self, input: GetWorkflowInput) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_workflow(input).await
    }

    /// List all available tools
    pub async fn execute_list_tools(&self, input: ListToolsInput) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_list_tools(input).await
    }

    /// Get details about a specific tool
    pub async fn execute_get_tool(&self, input: GetToolInput) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_get_tool(input).await
    }

    /// Connect to an external MCP server
    pub async fn execute_connect_mcp_server(&self, input: ConnectMcpServerInput) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_connect_mcp_server(input).await
    }

    /// Call a tool on a connected MCP server
    pub async fn execute_call_mcp_tool(&self, input: CallMcpToolInput) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        execute_call_mcp_tool(input).await
    }
}

impl ToolHandler for AgentToolsHandler {
    fn category(&self) -> &str {
        "agent"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "get_workflow".to_string(),
            "list_tools".to_string(),
            "get_tool".to_string(),
            "connect_mcp_server".to_string(),
            "call_tool".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "get_workflow",
                "Get workflow rules (must be called before other tools)",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "purpose": { "type": "string", "description": "Workflow purpose (default, general, etc.)" }
                    }
                })),
            ).with_title("Get Workflow"),
            rmcp::model::Tool::new(
                "list_tools",
                "List all available MCP tools",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "filter": { "type": "string", "description": "Filter tools by category" }
                    }
                })),
            ).with_title("List Tools"),
            rmcp::model::Tool::new(
                "get_tool",
                "Get details about a specific tool",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Tool name" }
                    },
                    "required": ["name"]
                })),
            ).with_title("Get Tool"),
            rmcp::model::Tool::new(
                "connect_mcp_server",
                "Connect to an external MCP server",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string", "description": "Server name" },
                        "command": { "type": "string", "description": "Command to run" },
                        "args": { "type": "array", "items": { "type": "string" }, "description": "Command arguments" }
                    },
                    "required": ["name", "command"]
                })),
            ).with_title("Connect MCP Server"),
            rmcp::model::Tool::new(
                "call_tool",
                "Call a tool on a connected MCP server",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tool_name": { "type": "string", "description": "Name of tool to call" },
                        "arguments": { "type": "string", "description": "JSON-encoded arguments" }
                    },
                    "required": ["tool_name"]
                })),
            ).with_title("Call MCP Tool"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "get_workflow" => {
                    let input: crate::bridge::tools::agent::inputs::GetWorkflowInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_workflow(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_tools" => {
                    let input: crate::bridge::tools::agent::inputs::ListToolsInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_list_tools(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_tool" => {
                    let input: crate::bridge::tools::agent::inputs::GetToolInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_tool(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "connect_mcp_server" => {
                    let input: crate::bridge::tools::agent::inputs::ConnectMcpServerInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_connect_mcp_server(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "call_tool" => {
                    let input: crate::bridge::tools::agent::inputs::CallMcpToolInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_call_mcp_tool(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string())),
            }
        }
    }
}
