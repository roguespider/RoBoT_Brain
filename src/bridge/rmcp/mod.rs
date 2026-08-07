// src/bridge/rmcp/mod.rs
// RMCP module - contains handler and tool definitions
//
// Architecture:
// - MCP loads first (ServerHandler impl is inlined here)
// - Each tool handler loads independently via ToolHandlerCollection
// - No single tool can cause MCP or any other tool to fail
// - Graceful degradation: if a handler fails, log warning but continue

pub mod types;
pub mod helpers;
pub mod handler;

pub use handler::run_stdio_server;

use rmcp::handler::server::ServerHandler;
use rmcp::model::{ServerCapabilities, ServerInfo, Implementation};

impl ServerHandler for types::McpServerHandler {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_experimental()
            .enable_extensions()
            .enable_completions()
            .enable_prompts()
            .enable_resources()
            .enable_tasks()
            .enable_tools()
            .enable_tool_list_changed()
            .build();

        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new(&self.name, &self.version))
    }
}
