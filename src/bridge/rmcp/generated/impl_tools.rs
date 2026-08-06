// impl_tools.rs - ServerHandler impl with custom server info
// This file is included by mod.rs

use rmcp::tool_handler;
use rmcp::model::ServerCapabilities;
use rmcp::model::Implementation;

#[tool_handler]
impl rmcp::handler::server::ServerHandler for McpServerHandler {
    fn get_info(&self) -> rmcp::model::ServerInfo {
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
            .with_server_info(Implementation::new(&self.name, &self.version))
    }
}
