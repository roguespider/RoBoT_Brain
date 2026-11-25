// src/bridge/mod.rs
//! Bridge layer — application assembly and the tool registry.
//!
//! The protocol surfaces (ACP, MCP, RMCP) live in [`crate::api`]; this module
//! owns what surrounds them: the application state and initialization
//! ([`app`]), the console/logging setup ([`logging`]) and the tool registry
//! that the MCP server publishes ([`tools`]).

pub mod app;
pub mod logging;
pub mod tools;

#[cfg(target_os = "windows")]
pub mod windows_console;
