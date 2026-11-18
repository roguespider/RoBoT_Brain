// src/api/mod.rs
//! API layer — protocol adapters that expose the brain to the outside world.
//!
//! Per v0.0.2 Appendix A the architecture places the bridge (application
//! assembly, tool registry, logging) inside the API layer.
//!
//! - [`acp`] — Agent Communication Protocol (inter-agent messaging, routing,
//!   capability negotiation).
//! - [`bridge`] — Application state, initialization, tool handlers, logging.
//! - [`mcp`] — Model Context Protocol (tool catalog, tool context, the
//!   per-tool handler collection).
//! - [`rmcp`] — the Rust MCP server implementation (stdio transport, the
//!   `ServerHandler` implementation, the workflow-enforcement gate).
//!
//! The MCP server is the process entry point: `rmcp::run_stdio_server` binds
//! the tool catalog to the transport and hands each call to the matching
//! `mcp::handlers` implementation.

pub mod acp;
pub mod bridge;
pub mod developer_interface;
pub mod mcp;
pub mod rmcp;
