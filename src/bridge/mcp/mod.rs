// src/bridge/mcp.rs
//! MCP (Model Context Protocol) module
//!
//! This module provides the MCP client implementation for connecting to
//! external MCP servers and handling MCP protocol communications.

pub mod client;
pub mod context;
pub mod handler;
pub mod types;

// Re-export commonly used types for backwards compatibility
pub use client::{McpClient, ToolError};
pub use context::McpContext;
pub use handler::{DefaultMcpHandler, McpHandler, ToolExecutor};
pub use types::{McpTool, McpCapabilities, McpClientInfo, McpServerInfo};
