// src/tools/agent.rs
// Agent-related MCP tools
// Module re-exports for backwards compatibility

pub mod definitions;
pub mod inputs;
pub mod mcp_tools;
pub mod workflows;

// Re-export only what's actually needed externally
pub use mcp_tools::{get_mcp_client, init_mcp_client};
