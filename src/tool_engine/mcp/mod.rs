//! MCP tool adapters — Per Architecture §13 (Tool Engine).
//! Model Context Protocol integration for exposing tools via MCP.

/// MCP tool server for exposing tools via the Model Context Protocol.
pub struct McpToolServer;

impl McpToolServer {
    /// Create a new MCP tool server.
    pub fn new() -> Self {
        Self
    }

    /// List available tools for MCP clients.
    pub fn list_tools(&self) -> Vec<String> {
        Vec::new()
    }
}
