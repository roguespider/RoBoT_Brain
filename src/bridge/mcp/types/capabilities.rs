//! MCP Capabilities Types
//!
//! Server capabilities for MCP protocol.

use serde::{Deserialize, Serialize};

/// Empty capability marker
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct McpEmpty;

impl std::fmt::Debug for McpEmpty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpEmpty").finish()
    }
}

/// Resources capability
#[derive(Clone, Serialize, Deserialize)]
pub struct McpResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

impl std::fmt::Debug for McpResourcesCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpResourcesCapability")
            .field("subscribe", &self.subscribe)
            .field("list_changed", &self.list_changed)
            .finish()
    }
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
#[derive(Clone, Serialize, Deserialize)]
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

impl std::fmt::Debug for McpCapabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpCapabilities")
            .field("tools", &self.tools)
            .field("resources", &self.resources)
            .field("prompts", &self.prompts)
            .field("logging", &self.logging)
            .finish()
    }
}

impl McpCapabilities {
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
