//! MCP Info Types
//!
//! Server information type for MCP protocol.

use serde::{Deserialize, Serialize};

/// Server information
#[derive(Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

impl std::fmt::Debug for McpServerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServerInfo")
            .field("name", &self.name)
            .field("version", &self.version)
            .finish()
    }
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
