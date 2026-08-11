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

        // Exercise check_multiple_tools to verify the enforcement gate can
        // batch-check the full tool catalog (Architecture §22).
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone().into_owned()).collect();
        let enforcement_result = self.enforcer.check_multiple_tools(&self.session_id, &tool_names).await;
        let enforcement_ok = enforcement_result.is_ok();
        tracing::debug!("batch enforcement check on {} tools: ok={}", tool_names.len(), enforcement_ok);

        // Clean up expired enforcement sessions on each tool listing so the
        // session table does not grow unbounded (Architecture §22).
        let removed = self.cleanup_expired_sessions().await;
        tracing::debug!("cleaned up {removed} expired enforcement session(s)");

        // Debug snapshot of the current session state for observability.
        if let Some(state) = self.get_session_state().await {
            tracing::debug!("session {} wf_retrieved={}", state.session_id, state.workflow_retrieved);
        }

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

        // Record explicit workflow-gate milestones for tools that mark them.
        // These calls use the dedicated ServerHandler wrappers which in turn
        // exercise the WorkflowEnforcer methods (Architecture §22).
        match tool_name {
            "get_workflow" => {
                let purpose = arguments.get("purpose")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                if let Some(p) = purpose {
                    self.update_workflow_purpose(p).await;
                }
            }
            "search_memory" | "query_knowledge" => {
                let query = arguments.get("query")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.record_memory_searched(query).await;
            }
            "get_patterns" | "analyze_patterns" => {
                self.record_patterns_reviewed().await;
            }
            _ => {}
        }

        // Try to execute via handler first
        match self.handlers.call_tool(tool_name, arguments.clone()).await {
            Ok(result) => {
                // Record tool execution for workflow tracking
                let query = arguments.get("query")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                self.record_tool_execution(tool_name, query).await;

                // Auto-record the tool outcome as an experience so the §4.04
                // learning spine advances without the caller manually
                // recording (Architecture §2.04, TASK-V2-05).
                let was_successful = result.success;
                self.emit_tool_experience(tool_name, was_successful, &arguments).await;

                // Build the response content via the shared helper, which
                // encodes both success and failure payloads consistently.
                let content = vec![crate::bridge::rmcp::helpers::tool_output_to_content(result)];
                if was_successful {
                    Ok(CallToolResult::success(content))
                } else {
                    Ok(CallToolResult::error(content))
                }
            }
            Err(err) => {
                // Auto-record the handler-level failure as an experience too
                // (Architecture §2.04, TASK-V2-05).
                self.emit_tool_experience(tool_name, false, &arguments).await;

                // Return tool-level error (not protocol error)
                let error_msg = match err {
                    HandlerError::ToolNotFound(name) => format!("Tool not found: {}", name),
                    HandlerError::ExecutionFailed(msg) => msg,
                    HandlerError::InvalidParams(msg) => format!("Invalid parameters: {}", msg),
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
