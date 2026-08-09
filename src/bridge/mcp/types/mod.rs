//! MCP Types Module
//!
//! These types define the MCP protocol for communication between
//! clients and servers.

pub mod capabilities;
pub mod error;
pub mod info;
pub mod message;
pub mod prompts;
pub mod resources;
pub mod self_check;
pub mod tools;

// Re-exports
pub use capabilities::{McpCapabilities, McpEmpty, McpResourcesCapability};
pub use info::{McpClientInfo, McpServerInfo};
pub use tools::McpTool;

use serde::{Deserialize, Serialize};

/// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    pub client_info: McpClientInfo,
}

impl InitializeParams {
    /// Create new initialize params
    pub fn new(client_info: McpClientInfo) -> Self {
        Self {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: McpCapabilities::default(),
            client_info,
        }
    }
}
