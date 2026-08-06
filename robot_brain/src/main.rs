//! RoBoT Brain MCP Server
//! 
//! A Rust MCP server that loads tools as runtime plugins.
//! If a plugin fails to load, the server continues with remaining plugins.

mod plugin_loader;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolRequest, CallToolResult, ContentBlock, ServerCapabilities, ServerInfo, Implementation};
use rmcp::tool::{Tool, ToolCall};
use rmcp::tool_handler;
use rmcp::tool_router;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use plugin_loader::PluginManager;

/// The MCP server handler that routes tool calls to plugins
pub struct McpServer {
    plugins: Arc<RwLock<PluginManager>>,
}

impl McpServer {
    pub fn new() -> Self {
        McpServer {
            plugins: Arc::new(RwLock::new(PluginManager::new())),
        }
    }

    /// Load plugins from a directory
    pub async fn load_plugins(&self, plugins_dir: PathBuf) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.load_from_directory(&plugins_dir)?;
        info!("Loaded {} plugins", plugins.count());
        Ok(())
    }

    /// Get all tools from loaded plugins
    pub fn get_tools(&self) -> Vec<Tool> {
        let plugins = self.plugins.blocking_read();
        plugins.all_tools()
            .into_iter()
            .map(|def| {
                Tool::new(def.name, def.description, def.input_schema)
            })
            .collect()
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl ServerHandler for Arc<McpServer> {
    async fn list_tools(&self) -> Vec<Tool> {
        self.get_tools()
    }

    async fn call_tool(&self, request: CallToolRequest) -> CallToolResult {
        let tool_name = &request.params.name;
        
        // Try to execute via plugins
        let plugins = self.plugins.read().await;
        match plugins.execute(tool_name, request.params.arguments.unwrap_or_default()) {
            Ok(result) => {
                CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: serde_json::to_string_pretty(&result).unwrap_or_default(),
                    }],
                    is_error: Some(false),
                }
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
                CallToolResult {
                    content: vec![ContentBlock::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                }
            }
        }
    }
}

#[tool_handler]
impl rmcp::handler::server::ServerHandler for Arc<McpServer> {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_tool_list_changed()
            .build();

        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new("robot_brain", "0.0.1"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting RoBoT Brain MCP Server...");

    // Create server
    let server = Arc::new(McpServer::new());

    // Load plugins from default directory
    let plugins_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("plugins");
    
    // Create plugins directory if it doesn't exist
    if !plugins_dir.exists() {
        std::fs::create_dir_all(&plugins_dir)?;
        info!("Created plugins directory: {:?}", plugins_dir);
        info!("Build tool crates and copy .so files to plugins directory");
    }

    // Try to load plugins (will warn if none found)
    if let Err(e) = server.load_plugins(plugins_dir.clone()).await {
        tracing::warn!("No plugins loaded: {}", e);
        tracing::warn!("Build tool crates and copy .so files to: {:?}", plugins_dir);
    }

    info!("RoBoT Brain MCP Server initialized");
    info!("Loaded {} plugins", server.plugins.read().await.count());

    // TODO: Start actual MCP server transport
    // For now, just demonstrate that the plugin system works
    println!("\n=== RoBoT Brain MCP Server ===");
    println!("Plugins loaded: {}", server.plugins.read().await.count());
    println!("Tools available: {}", server.get_tools().len());
    println!("\nTool list:");
    for tool in server.get_tools() {
        println!("  - {}", tool.name);
    }

    Ok(())
}
