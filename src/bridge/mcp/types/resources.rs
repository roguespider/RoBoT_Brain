//! MCP Resource Types
//!
//! Resource definitions for MCP protocol.

use serde::{Deserialize, Serialize};

/// Resource definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

impl McpResource {
    /// Create a new resource
    pub fn new(uri: &str, name: &str) -> Self {
        Self {
            uri: uri.to_string(),
            name: name.to_string(),
            description: None,
            mime_type: None,
        }
    }
}
