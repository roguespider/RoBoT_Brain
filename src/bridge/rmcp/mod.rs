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

use std::collections::HashMap;
use std::sync::Arc;

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
        if let Err(e) = self.enforcer.check_enforcement(&self.session_id, tool_name).await {
            let error_msg = serde_json::to_string(&e).unwrap_or_else(|_| e.message.clone());
            let content = vec![ContentBlock::text(error_msg)];
            return Ok(CallToolResult::error(content));
        }

        // Try to execute via handler first
        match self.handlers.call_tool(tool_name, arguments.clone()).await {
            Ok(result) => {
                // Record tool execution for workflow tracking
                let query = arguments.get("query")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.enforcer.record_tool_execution(&self.session_id, tool_name, query).await;
                
                // Check if the tool considers itself successful
                if result.success {
                    // Return the tool's data directly so validation can find expected fields
                    let json_str = serde_json::to_string(&result.data)
                        .unwrap_or_else(|_| result.data.to_string());
                    let content = vec![ContentBlock::text(json_str)];
                    Ok(CallToolResult::success(content))
                } else {
                    // Tool returned success=false, treat as error response
                    let error_msg = result.error.clone().unwrap_or_else(|| "Unknown error".to_string());
                    let error_response = serde_json::json!({
                        "success": false,
                        "data": result.data,
                        "error": error_msg
                    });
                    let json_str = serde_json::to_string(&error_response)
                        .unwrap_or_else(|_| r#"{"success": false, "error": "Failed to serialize error"}"#.to_string());
                    let content = vec![ContentBlock::text(json_str)];
                    Ok(CallToolResult::error(content))
                }
            }
            Err(err) => {
                // Return tool-level error (not protocol error)
                let error_msg = match err {
                    HandlerError::ToolNotFound(name) => format!("Tool not found: {}", name),
                    HandlerError::ExecutionFailed(msg) => msg,
                    HandlerError::InvalidParams(msg) => format!("Invalid parameters: {}", msg),
                    HandlerError::HandlerNotFound(category) => format!("Handler not found for category: {}", category),
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
