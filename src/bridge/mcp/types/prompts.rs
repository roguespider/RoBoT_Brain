//! MCP Prompt Types
//!
//! Prompt definitions for MCP protocol.

use serde::{Deserialize, Serialize};

/// Prompt definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

/// Argument for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

impl McpPromptArgument {
    /// Create a new prompt argument
    pub fn new(name: &str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            required,
        }
    }
}
