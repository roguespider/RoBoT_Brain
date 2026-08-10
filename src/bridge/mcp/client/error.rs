// src/bridge/mcp/client/error.rs
//! Tool error types for MCP client

/// Tool invocation error
#[derive(Debug, Clone)]
pub struct ToolError {
    pub message: String,
    pub server: String,
    pub tool: String,
}

impl ToolError {
    /// Create a new tool error
    pub fn new(server: &str, tool: &str, message: &str) -> Self {
        Self {
            message: message.to_string(),
            server: server.to_string(),
            tool: tool.to_string(),
        }
    }

    /// Create an error for tool not found
    pub fn not_found(tool: &str) -> Self {
        Self {
            message: format!("Tool '{}' not found on any connected server", tool),
            server: "unknown".to_string(),
            tool: tool.to_string(),
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.server, self.tool, self.message)
    }
}

impl std::error::Error for ToolError {}
