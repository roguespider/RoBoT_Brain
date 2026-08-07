// src/bridge/rmcp/mod.rs
// RMCP module - contains handler and tool definitions
//
// Architecture:
// - MCP loads first (server_handler_impl.rs via generated module)
// - Each tool handler loads independently via ToolHandlerCollection
// - No single tool can cause MCP or any other tool to fail
// - Graceful degradation: if a handler fails, log warning but continue

pub mod types;
pub mod helpers;
pub mod handler;
pub mod generated;

pub use handler::run_stdio_server;
