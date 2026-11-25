//! MCP Server Manager — Per Architecture Chapter 13 (Tool Engine).
//!
//! Manages MCP server connections and lifecycle.

use crate::tool_engine::mcp::McpToolServer;

/// Manager for MCP server connections.
pub struct McpServerManager {
    servers: Vec<McpToolServer>,
}

impl McpServerManager {
    /// Create a new MCP server manager.
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
        }
    }

    /// Add an MCP server to the manager.
    pub fn add_server(&mut self, server: McpToolServer) {
        self.servers.push(server);
    }

    /// Get the number of managed servers.
    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    /// List all managed server tools.
    pub fn list_all_tools(&self) -> Vec<String> {
        self.servers.iter().flat_map(|s| s.list_tools()).collect()
    }
}

impl Default for McpServerManager {
    fn default() -> Self {
        Self::new()
    }
}
