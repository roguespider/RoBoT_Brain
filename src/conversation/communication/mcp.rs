//! MCP integration - Per Architecture §15.1 "MCP integration"

use serde::{Deserialize, Serialize};

/// Tool descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDescriptor {
    /// Tool name.
    pub name: String,
    /// Description.
    pub description: String,
}

/// MCP error.
#[derive(Debug, Clone, PartialEq)]
pub enum McpError {
    ToolNotFound(String),
    InvalidArgs(String),
}

/// MCP request.
#[derive(Debug, Clone, PartialEq)]
pub struct McpRequest {
    /// Tool name.
    pub tool_name: String,
    /// Arguments.
    pub args: serde_json::Value,
}

/// MCP response.
#[derive(Debug, Clone, PartialEq)]
pub struct McpResponse {
    /// Result content.
    pub content: Vec<serde_json::Value>,
    /// Whether this is an error.
    pub is_error: bool,
}

/// MCP handler trait.
pub trait McpHandler {
    /// List available tools.
    fn list_tools(&self) -> Vec<ToolDescriptor>;
    /// Call a tool by name with arguments.
    fn call_tool(&self, name: &str, args: serde_json::Value)
    -> Result<serde_json::Value, McpError>;
}

/// Dispatch a request to an MCP handler.
pub fn dispatch_to_mcp(server: &dyn McpHandler, req: McpRequest) -> McpResponse {
    match server.call_tool(&req.tool_name, req.args) {
        Ok(result) => McpResponse {
            content: vec![result],
            is_error: false,
        },
        Err(_) => McpResponse {
            content: vec![serde_json::json!({"error": "execution failed"})],
            is_error: true,
        },
    }
}

