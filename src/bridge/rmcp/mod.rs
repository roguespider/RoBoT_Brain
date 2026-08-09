// src/bridge/rmcp/mod.rs
// RMCP module - contains handler and tool definitions
//
// Architecture:
// - MCP loads first (ServerHandler impl is inlined here)
// - Each tool handler loads independently via ToolHandlerCollection
// - No single tool can cause MCP or any other tool to fail
// - Graceful degradation: if a handler fails, log warning but continue

pub mod types;
pub mod helpers;
pub mod handler;

pub use handler::run_stdio_server;

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, 
    ServerInfo, Implementation, Tool
};
use rmcp::service::RequestContext;
use rmcp::ErrorData;

use crate::bridge::mcp::handlers::HandlerError;

impl ServerHandler for types::McpServerHandler {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_experimental()
            .enable_extensions()
            .enable_completions()
            .enable_prompts()
            .enable_resources()
            .enable_tasks()
            .enable_tools()
            .enable_tool_list_changed()
            .build();

        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new(&self.name, &self.version))
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = self.handlers.get_all_tools();
        Ok(ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _: RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name: &str = &request.name;
        let arguments = request.arguments.map(|args| serde_json::Value::Object(args))
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        // Check workflow enforcement FIRST - agent MUST follow workflows
        if let Err(e) = self.check_workflow_enforcement(tool_name).await {
            let content = vec![crate::bridge::rmcp::helpers::enforcement_error_to_content(e)];
            return Ok(CallToolResult::error(content));
        }

        // Try to execute via handler first
        match self.handlers.call_tool(tool_name, arguments.clone()).await {
            Ok(result) => {
                // Record tool execution for workflow tracking
                let query = arguments.get("query")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.record_tool_execution(tool_name, query).await;
                
                // Build the response content via the shared helper, which
                // encodes both success and failure payloads consistently.
                let was_successful = result.success;
                let content = vec![crate::bridge::rmcp::helpers::tool_output_to_content(result)];
                if was_successful {
                    Ok(CallToolResult::success(content))
                } else {
                    Ok(CallToolResult::error(content))
                }
            }
            Err(err) => {
                // Return tool-level error (not protocol error)
                let error_msg = match err {
                    HandlerError::ToolNotFound(name) => format!("Tool not found: {}", name),
                    HandlerError::ExecutionFailed(msg) => msg,
                    HandlerError::InvalidParams(msg) => format!("Invalid parameters: {}", msg),
                    HandlerError::HandlerNotFound(category) => {
                        let available = self.is_handler_available(&category);
                        format!(
                            "Handler not found for category: {} (available: {})",
                            category, available
                        )
                    },
                    HandlerError::Internal(msg) => format!("Internal error: {}", msg),
                };
                let content = vec![ContentBlock::text(error_msg)];
                Ok(CallToolResult::error(content))
            }
        }
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.handlers.get_tool(name)
    }
}
