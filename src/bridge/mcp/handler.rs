// src/bridge/mcp/handler.rs

//! MCP protocol handler implementation
//!
//! This module provides the handler implementation for processing
//! MCP protocol requests and notifications.

use anyhow::Result;
use std::sync::Arc;

use super::types::{McpCapabilities, McpNotification, McpRequest, McpResponse, McpServerInfo};

/// Trait for MCP protocol handlers (defined for future extensibility)
pub trait McpHandler: Send + Sync {
    /// Handle an MCP request
    fn handle_request(&self, request: McpRequest) -> Result<McpResponse>;
    /// Handle an MCP notification
    fn handle_notification(&self, notification: McpNotification) -> Result<()>;
    /// Get server capabilities
    fn get_capabilities(&self) -> McpCapabilities;
    /// Get server info
    fn get_server_info(&self) -> McpServerInfo;
}

/// Tool executor trait for handling tool calls
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool with the given name and arguments
    fn execute(&self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value>;
}

/// Default tool executor that returns an error (to be extended)
pub struct DefaultToolExecutor;

impl ToolExecutor for DefaultToolExecutor {
    fn execute(&self, tool_name: &str, _arguments: serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "error": format!("Tool '{}' not found", tool_name)
        }))
    }
}

/// Default MCP handler that provides basic request routing
pub struct DefaultMcpHandler {
    capabilities: McpCapabilities,
    server_info: McpServerInfo,
    tool_executor: Arc<dyn ToolExecutor>,
    tools: Vec<super::types::McpTool>,
}

impl DefaultMcpHandler {
    /// Create a new default handler with the given info
    pub fn new(server_name: &str, server_version: &str) -> Self {
        Self {
            capabilities: McpCapabilities {
                tools: Some(super::types::McpEmpty),
                resources: None,
                prompts: None,
                logging: Some(super::types::McpEmpty),
            },
            server_info: McpServerInfo {
                name: server_name.to_string(),
                version: server_version.to_string(),
            },
            tool_executor: Arc::new(DefaultToolExecutor),
            tools: Vec::new(),
        }
    }

    /// Create a handler with a custom tool executor
    pub fn with_executor(
        server_name: &str,
        server_version: &str,
        executor: Arc<dyn ToolExecutor>,
    ) -> Self {
        Self {
            capabilities: McpCapabilities::with_tools(),
            server_info: McpServerInfo {
                name: server_name.to_string(),
                version: server_version.to_string(),
            },
            tool_executor: executor,
            tools: Vec::new(),
        }
    }

    /// Add a tool to the handler
    pub fn add_tool(&mut self, tool: super::types::McpTool) {
        self.tools.push(tool);
    }

    /// Set the tools list
    pub fn set_tools(&mut self, tools: Vec<super::types::McpTool>) {
        self.tools = tools;
    }

    /// Get the list of tools
    pub fn get_tools(&self) -> &[super::types::McpTool] {
        &self.tools
    }

    /// Handle a tool call request
    fn handle_tool_call(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
        self.tool_executor.execute(tool_name, arguments)
    }

    /// Handle a list tools request
    fn handle_list_tools(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "tools": self.tools.iter().map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            }).collect::<Vec<_>>()
        }))
    }

    /// Handle initialize request
    fn handle_initialize(&self, _params: serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "protocolVersion": super::types::MCP_VERSION,
            "capabilities": self.capabilities,
            "serverInfo": self.server_info
        }))
    }
}

impl McpHandler for DefaultMcpHandler {
    fn handle_request(&self, request: McpRequest) -> Result<McpResponse> {
        tracing::debug!("Handling MCP request: {}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params.unwrap_or(serde_json::Value::Null)),
            "tools/list" => self.handle_list_tools(),
            "tools/call" => {
                let params = request.params.unwrap_or(serde_json::Value::Null);
                self.handle_tool_call(params)
            }
            "ping" => Ok(serde_json::json!({"pong": true})),
            _ => Ok(serde_json::json!({
                "error": format!("Unknown method: {}", request.method)
            })),
        };

        match result {
            Ok(value) => Ok(McpResponse::success(&request.id, value)),
            Err(e) => Ok(McpResponse::error(
                &request.id,
                super::types::McpError::new(-32603, &e.to_string()),
            )),
        }
    }

    fn handle_notification(&self, notification: McpNotification) -> Result<()> {
        tracing::debug!("Received MCP notification: {}", notification.method);
        Ok(())
    }

    fn get_capabilities(&self) -> McpCapabilities {
        self.capabilities.clone()
    }

    fn get_server_info(&self) -> McpServerInfo {
        self.server_info.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_handler_creation() {
        let handler = DefaultMcpHandler::new("test-server", "1.0.0");
        assert_eq!(handler.get_server_info().name, "test-server");
        assert_eq!(handler.get_server_info().version, "1.0.0");
        assert!(handler.get_tools().is_empty());
    }

    #[test]
    fn test_handler_with_tools() {
        let mut handler = DefaultMcpHandler::new("test-server", "1.0.0");
        handler.add_tool(super::super::types::McpTool::new("test_tool", "A test tool"));
        assert_eq!(handler.get_tools().len(), 1);
    }

    #[test]
    fn test_handle_list_tools() {
        let mut handler = DefaultMcpHandler::new("test-server", "1.0.0");
        handler.add_tool(super::super::types::McpTool::new("tool1", "First tool"));
        handler.add_tool(super::super::types::McpTool::new("tool2", "Second tool"));

        let result = handler.handle_list_tools().unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_handle_request_initialize() {
        let handler = DefaultMcpHandler::new("test-server", "1.0.0");
        let request = McpRequest::new("initialize", "1");
        
        let response = handler.handle_request(request).unwrap();
        assert!(response.is_success());
        let result = response.result.unwrap();
        assert!(result.get("serverInfo").is_some());
    }

    #[test]
    fn test_handle_request_tools_list() {
        let handler = DefaultMcpHandler::new("test-server", "1.0.0");
        let request = McpRequest::new("tools/list", "2");
        
        let response = handler.handle_request(request).unwrap();
        assert!(response.is_success());
    }

    #[test]
    fn test_handle_request_unknown_method() {
        let handler = DefaultMcpHandler::new("test-server", "1.0.0");
        let request = McpRequest::new("unknown/method", "3");
        
        let response = handler.handle_request(request).unwrap();
        assert!(response.is_success());
        let result = response.result.unwrap();
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_handle_request_ping() {
        let handler = DefaultMcpHandler::new("test-server", "1.0.0");
        let request = McpRequest::new("ping", "4");
        
        let response = handler.handle_request(request).unwrap();
        assert!(response.is_success());
        assert_eq!(response.result.unwrap(), serde_json::json!({"pong": true}));
    }

    #[test]
    fn test_handle_notification() {
        let handler = DefaultMcpHandler::new("test-server", "1.0.0");
        let notification = McpNotification::new("test/notification");
        
        let result = handler.handle_notification(notification);
        assert!(result.is_ok());
    }
}
