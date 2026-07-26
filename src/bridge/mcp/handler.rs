// src/bridge/mcp/handler.rs
// MCP protocol handler trait

use anyhow::Result;

use super::types::{McpCapabilities, McpNotification, McpRequest, McpResponse, McpServerInfo};

/// Trait for MCP protocol handlers (reserved for future use)
#[allow(dead_code)]
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
