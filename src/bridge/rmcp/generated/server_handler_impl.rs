// server_handler_impl.rs
// Implements ServerHandler trait for McpServerHandler
// This is separate from tool implementations to allow tools to load independently.

use crate::bridge::rmcp::types::McpServerHandler;
use rmcp::handler::server::ServerHandler;

impl ServerHandler for McpServerHandler {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        use rmcp::model::ServerCapabilities;

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

        rmcp::model::ServerInfo::new(capabilities)
            .with_server_info(rmcp::model::Implementation::new(&self.name, &self.version))
    }
}
