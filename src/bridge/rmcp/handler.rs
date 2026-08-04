// src/bridge/rmcp/handler.rs
// RMCP stdio server handler

use std::sync::Arc;

use anyhow::Result;
use rmcp::serve_server;

use crate::bridge::mcp::McpContext;
use crate::bridge::rmcp::types::McpServerHandler;

/// Create a new RMCP server with stdio transport
pub async fn run_stdio_server(name: &str, version: &str, context: Arc<McpContext>) -> Result<()> {
    // Initialize stderr logging via tracing-subscriber (replaces deprecated MCP logging)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init() // Use try_init to avoid panic if logging.rs already set one
        .ok(); // Silently ignore if subscriber is already configured

    tracing::info!(
        "Starting RMCP server '{}' v{} with stdio transport",
        name,
        version
    );

    let mut handler = McpServerHandler::new(context, name.to_string(), version.to_string());
    handler.session_id = handler.new_session();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    eprintln!("DEBUG: About to call serve_server");

    let running = serve_server(handler, (stdin, stdout)).await?;
    eprintln!("DEBUG: serve_server returned");
    eprintln!("DEBUG: Server is now listening for messages...");

    tracing::info!("Server started, waiting for connections...");
    let quit_reason = running.waiting().await?;
    tracing::info!("Server stopped: {:?}", quit_reason);

    Ok(())
}
