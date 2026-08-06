// src/bridge/rmcp/generated/mod.rs
// Tools module - each tool category is a separate, isolated module
//
// Architecture:
// - server_handler_impl.rs: ServerHandler trait impl for MCP core (loads first)
// - tools_impl.rs: All tool implementations with #[tool_router] and #[tool] macros
// - If one tool has compile errors, disable its feature in Cargo.toml
// - MCP loads first (server_handler_impl.rs), tools load independently

mod server_handler_impl;
include!("tools_impl.rs");
