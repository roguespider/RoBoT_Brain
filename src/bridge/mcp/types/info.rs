//! MCP Info Types
//!
//! Server information type for MCP protocol.

use serde::{Deserialize, Serialize};

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

impl McpServerInfo {
    /// Create new server info
    pub fn from_name_version(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}
