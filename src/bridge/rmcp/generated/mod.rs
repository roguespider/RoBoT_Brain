// src/bridge/rmcp/generated/mod.rs
// Tools module - MCP core handler implementation
//
// Architecture:
// - server_handler_impl.rs: ServerHandler trait impl for MCP core (loads first)
// - Tool implementations have been moved to src/bridge/tools/handlers/
// - Each tool category has its own handler with isolated initialization
// - Graceful degradation: if a handler fails to init, log warning but continue
// - MCP loads first, then each tool handler loads independently

mod server_handler_impl;
