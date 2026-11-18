// src/bridge/mcp.rs
//! MCP (Model Context Protocol) module
//!
//! This module provides the MCP client implementation for connecting to
//! external MCP servers and handling MCP protocol communications.

pub mod client;
pub mod context;
pub mod handlers;  // HOW tools respond to MCP protocol
pub mod types;

// Re-export commonly used types
pub use client::McpClient;
pub use context::McpContext;
pub use types::McpTool;
