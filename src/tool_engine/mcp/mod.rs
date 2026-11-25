//! MCP tool adapters — Per Architecture §13 (Tool Engine).
//! Model Context Protocol integration for exposing tools via MCP.

pub mod client;
pub mod discovery;
pub mod manager;
pub mod registry;

/// MCP tool server for exposing tools via the Model Context Protocol.
#[derive(Default)]
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

// Re-export MCP subsystem types
pub use client::McpClient;
pub use discovery::McpDiscovery;
pub use manager::McpServerManager;
pub use registry::McpToolRegistry;
