// src/bridge/mcp.rs
//! MCP (Model Context Protocol) module
//!
//! This module provides the MCP client implementation for connecting to
//! external MCP servers and handling MCP protocol communications.

pub mod client;
pub mod context;
pub mod types;

// Re-export commonly used types
pub use client::McpClient;
#[allow(unused_imports)]
pub use client::ToolError;
pub use context::McpContext;
#[allow(unused_imports)]
pub use types::{McpTool, McpCapabilities, McpClientInfo, McpServerInfo};
