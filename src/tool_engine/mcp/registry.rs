//! MCP Tool Registry — Per Architecture Chapter 13 (Tool Engine).
//!
//! Registry for MCP tools.

/// Registry for MCP tools.
pub struct McpToolRegistry {
    tools: Vec<String>,
}

impl McpToolRegistry {
    /// Create a new MCP tool registry.
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool in the registry.
    pub fn register_tool(&mut self, tool: String) {
        self.tools.push(tool);
    }

    /// List all registered tools.
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.clone()
    }

    /// Check if a tool is registered.
    pub fn has_tool(&self, tool: &str) -> bool {
        self.tools.iter().any(|t| t == tool)
    }
}

impl Default for McpToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
