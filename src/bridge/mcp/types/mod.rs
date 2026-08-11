//! MCP Types Module
//!
//! These types define the MCP protocol for communication between
//! clients and servers.

pub mod capabilities;
pub mod info;
pub mod tools;

// Re-exports
pub use capabilities::McpCapabilities;
pub use info::McpServerInfo;
pub use tools::McpTool;
