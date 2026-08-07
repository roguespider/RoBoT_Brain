// src/bridge/tools/handlers/agent_handler.rs
// Agent tools handler - handles MCP connection and workflow tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::agent::{inputs::*, mcp_tools::*, workflows::*};
use crate::bridge::mcp::handlers::{HandlerInitError, HandlerInitResult, ToolHandler};
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
}
