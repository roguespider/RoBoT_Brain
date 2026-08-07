//! MCP Capabilities Types
//!
//! Server capabilities for MCP protocol.

use serde::{Deserialize, Serialize};

/// Empty capability marker
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpEmpty;

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

impl Default for McpResourcesCapability {
    fn default() -> Self {
        Self {
            subscribe: Some(true),
            list_changed: Some(true),
        }
    }
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    #[serde(default)]
    pub tools: Option<McpEmpty>,
    #[serde(default)]
    pub resources: Option<McpResourcesCapability>,
    #[serde(default)]
    pub prompts: Option<McpEmpty>,
    #[serde(default)]
    pub logging: Option<McpEmpty>,
}

impl McpCapabilities {
    /// Create capabilities with tools enabled
    pub fn with_tools() -> Self {
        Self {
            tools: Some(McpEmpty),
            resources: None,
            prompts: None,
            logging: None,
        }
    }

    /// Create capabilities with all features enabled
    pub fn all() -> Self {
        Self {
            tools: Some(McpEmpty),
            resources: Some(McpResourcesCapability::default()),
            prompts: Some(McpEmpty),
            logging: Some(McpEmpty),
        }
    }
}

impl Default for McpCapabilities {
    fn default() -> Self {
        Self::all()
    }
}
