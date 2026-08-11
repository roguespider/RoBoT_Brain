//! MCP Tool Types
//!
//! Tool definitions for MCP protocol.

use serde::{Deserialize, Serialize};

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
