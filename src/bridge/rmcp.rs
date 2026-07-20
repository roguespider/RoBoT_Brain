// src/bridge/rmcp.rs
// RMCP (Rust MCP) server implementation using the rmcp crate

use anyhow::Result;
use rmcp::{
    ServerHandler,
    service::serve_server,
};
use std::sync::Arc;

use super::mcp::McpContext;

/// RMCP server wrapper for MCP bridge
pub struct RmcpServer {
    context: Arc<McpContext>,
}

/// Handler implementation for MCP requests
pub struct RmcpHandler {
    pub name: String,
    pub version: String,
}

impl RmcpHandler {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
        }
    }
}

impl ServerHandler for RmcpHandler {}

impl RmcpServer {
    /// Get the shared context
    pub fn context(&self) -> Arc<McpContext> {
        Arc::clone(&self.context)
    }
}

/// Create a new RMCP server with stdio transport
pub async fn run_stdio_server(name: &str, version: &str) -> Result<()> {
    tracing::info!("Starting RMCP server with stdio transport");
    
    let handler = RmcpHandler::new(name, version);
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    
    serve_server(handler, (stdin, stdout)).await?;
    
    Ok(())
}

/// Default RMCP server factory
pub fn create_rmcp_server() -> impl Fn() -> RmcpHandler {
    || RmcpHandler::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}
