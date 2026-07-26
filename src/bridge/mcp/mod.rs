// src/bridge/mcp.rs
// MCP (Model Context Protocol) module
// Re-exports all MCP types and implementations

#![allow(dead_code)]

pub mod client;
pub mod context;
pub mod handler;
pub mod types;

// Re-export commonly used types for backwards compatibility
pub use client::McpClient;
pub use context::McpContext;
pub use types::McpTool;
