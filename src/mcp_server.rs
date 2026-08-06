//! RoBoT Brain MCP Server
//! 
//! MCP server implementation that loads tools as runtime plugins.
//! If a plugin fails to load, the server continues with remaining plugins.
//!
//! Supports MCP Protocol 2025-03-26:
//! - Streamable HTTP transport
//! - OAuth 2.1 authorization framework
//! - Tool annotations
//! - Argument completions

use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, ListToolsResult, 
    ServerCapabilities, ServerInfo, Implementation, TextContent, Tool, 
    PaginatedRequestParams, ProtocolVersion
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;
use rmcp::service::MaybeSendFuture;
use serde_json::Map;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use crate::plugin_loader::PluginManager;

/// MCP Protocol version supported by this server
const SUPPORTED_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::LATEST;

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
    pub async fn load_plugins(&self, plugins_dir: PathBuf) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut plugins = self.plugins.write().await;
        plugins.load_from_directory(&plugins_dir)?;
        info!("Loaded {} plugins", plugins.count());
        Ok(())
    }

    /// Convert serde_json::Value to Arc<Map<String, serde_json::Value>>
    fn value_to_schema(value: serde_json::Value) -> Arc<Map<String, serde_json::Value>> {
        match value {
            serde_json::Value::Object(map) => Arc::new(map),
            _ => Arc::new(Map::new()),
        }
    }

    /// Get all tools from loaded plugins
    pub fn get_tools(&self) -> Vec<Tool> {
        let plugins = self.plugins.blocking_read();
        plugins.all_tools()
            .into_iter()
            .map(|def| {
                let schema = Self::value_to_schema(def.input_schema);
                // Use the newer Tool builder pattern for MCP 2025-03-26+
                Tool::new(def.name, def.description, schema)
            })
            .collect()
    }

    /// Execute a tool call
    pub async fn execute_tool(&self, params: CallToolRequestParams) -> Result<CallToolResult, McpError> {
        let tool_name = &params.name;
        
        // Try to execute via plugins
        let plugins = self.plugins.read().await;
        
        // Get arguments as serde_json::Value
        let arguments = match params.arguments {
            Some(args) => serde_json::Value::Object(args),
            None => serde_json::Value::Object(serde_json::Map::new()),
        };
        
        match plugins.execute(tool_name, arguments) {
            Ok(result) => {
                match serde_json::to_string_pretty(&result) {
                    Ok(text) => Ok(CallToolResult::success(vec![ContentBlock::Text(TextContent::new(text))])),
                    Err(serde_err) => {
                        let err_msg = format!("Failed to serialize result: {}", serde_err);
                        error!("{}", err_msg);
                        Ok(CallToolResult::error(vec![ContentBlock::Text(TextContent::new(err_msg))]))
                    }
                }
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
                Ok(CallToolResult::error(vec![ContentBlock::Text(TextContent::new(format!("Error: {}", e)))]))
            }
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper that implements ServerHandler for our server
/// Updated for MCP 2025-03-26 protocol
pub struct McpServerHandler {
    server: Arc<McpServer>,
}

impl McpServerHandler {
    pub fn new(server: Arc<McpServer>) -> Self {
        McpServerHandler { server }
    }
}

impl ServerHandler for McpServerHandler {
    /// Get server info with supported protocol versions
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_tool_list_changed()
            .build();

        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new("robot_brain", "0.0.1"))
            .with_protocol_version(SUPPORTED_PROTOCOL_VERSION)
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        let tools = self.server.get_tools();
        std::future::ready(Ok(ListToolsResult::with_all_items(tools)))
    }

    fn call_tool(
        &self,
        params: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + MaybeSendFuture + '_ {
        let server = self.server.clone();
        async move {
            server.execute_tool(params).await
        }
    }
}

/// Initialize and start the MCP server
pub async fn run_mcp_server() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting RoBoT Brain MCP Server...");
    info!("MCP Protocol: 2025-03-26 (Streamable HTTP, Tool Annotations)");

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
    match server.load_plugins(plugins_dir.clone()).await {
        Ok(()) => {
            info!("RoBoT Brain MCP Server initialized");
            info!("Loaded {} plugins", server.plugins.read().await.count());
        }
        Err(e) => {
            warn!("No plugins loaded: {}", e);
            warn!("Build tool crates and copy .so files to: {:?}", plugins_dir);
        }
    }

    println!("\n=== RoBoT Brain MCP Server ===");
    println!("MCP Protocol: 2025-03-26");
    println!("Plugins loaded: {}", server.plugins.read().await.count());
    println!("Tools available: {}", server.get_tools().len());
    println!("\nTool list:");
    for tool in server.get_tools() {
        println!("  - {}", tool.name);
    }

    Ok(())
}
