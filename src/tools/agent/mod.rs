// src/tools/agent.rs
// Agent-related MCP tools
// Module re-exports for backwards compatibility

#![allow(unused_imports)]

pub mod definitions;
pub mod inputs;
pub mod mcp_tools;
pub mod workflows;

// Re-export for backwards compatibility
pub use inputs::{
    CallMcpToolInput, ConnectMcpServerInput, GetToolInput, GetWorkflowInput, ListToolsInput,
};
pub use mcp_tools::{
    execute_call_mcp_tool, execute_connect_mcp_server, execute_get_tool, init_mcp_client,
};
pub use workflows::{execute_get_workflow, execute_list_tools};
