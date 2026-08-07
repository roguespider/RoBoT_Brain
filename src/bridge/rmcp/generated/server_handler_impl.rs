// server_handler_impl.rs
// Implements ServerHandler trait for McpServerHandler
//
// Load order:
// 1. MCP core loads first (this file)
// 2. Each tool handler loads independently via ToolHandlerCollection
// 3. No single tool can cause MCP or any other tool to fail

use crate::bridge::rmcp::types::McpServerHandler;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{ServerCapabilities, ServerInfo, Implementation};

impl ServerHandler for McpServerHandler {
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

// Tool implementations are now in handlers/
// Each tool handler has its own implementation for its local type
// McpServerHandler delegates to the appropriate handler based on tool name
