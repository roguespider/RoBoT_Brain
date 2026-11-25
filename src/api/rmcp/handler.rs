// src/api/rmcp/handler.rs
// RMCP stdio server handler
// Supports MCP Protocol 2025-03-26

use std::io::{self, BufRead};
use std::sync::Arc;

use anyhow::Result;
use rmcp::ServerHandler;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use crate::api::mcp::McpContext;
use crate::api::rmcp::types::McpServerHandler;

/// Create a new RMCP server with stdio transport
pub async fn run_stdio_server(name: &str, version: &str, context: Arc<McpContext>) -> Result<()> {
    // Initialize stderr logging via tracing-subscriber (replaces deprecated MCP logging)
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(env_filter);
    subscriber.try_init().unwrap_or_else(|e| {
        tracing::warn!("Failed to initialize tracing subscriber: {e}");
    });

    eprintln!("[RoBoT] Starting MCP server '{}' v{}", name, version);
    eprintln!("[RoBoT] MCP Protocol: 2025-03-26");
    tracing::info!(
        "Starting RMCP server '{}' v{} with stdio transport",
        name,
        version
    );
    tracing::info!("MCP Protocol: 2025-03-26 (Streamable HTTP, Tool Annotations)");

    let mut handler = McpServerHandler::new(context, name.to_string(), version.to_string());
    handler.session_id = handler.new_session();

    // Initialize all tool handlers with graceful degradation
    handler.initialize_handlers();

    let total_tools = handler.total_tools();
    let healthy = handler.is_healthy();

    eprintln!(
        "[RoBoT] Tool handlers initialized: {} tools available, healthy: {}",
        total_tools, healthy
    );
    tracing::info!(
        "Tool handlers initialized: {} tools available, healthy: {}",
        total_tools,
        healthy
    );

    // Log any handler initialization errors
    if !handler.handler_errors.is_empty() {
        for err in &handler.handler_errors {
            tracing::warn!("Handler init error [{}]: {}", err.category, err.message);
        }
    }

    // Use blocking stdin in a background thread because tokio::io::stdin()
    // does not work reliably with pipe input on Windows.
    let (tx, mut rx) = mpsc::channel::<String>(128);
    let stdin_thread = std::thread::spawn(move || {
        let stdin = io::stdin();
        let locked = stdin.lock();
        for line in locked.lines() {
            match line {
                Ok(l) => {
                    if tx.blocking_send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    eprintln!("[RoBoT] Server ready, listening on stdio...");
    tracing::info!("Server ready, listening on stdio...");

    let mut stdout = tokio::io::stdout();

    while let Some(line) = rx.recv().await {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse JSON-RPC request
        let request: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let id = request.get("id").cloned();
        let params = request
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        match method {
            "initialize" => {
                let info = handler.get_info();
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": info.capabilities,
                        "serverInfo": {
                            "name": info.server_info.name,
                            "version": info.server_info.version,
                        },
                    },
                });
                let response_str = serde_json::to_string(&response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            "notifications/initialized" => {
                // No response needed for notifications
            }
            "tools/list" => {
                let tools = handler.handlers.get_all_tools();
                let tools_json: Vec<serde_json::Value> = tools
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema,
                        })
                    })
                    .collect();
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": tools_json,
                    },
                });
                let response_str = serde_json::to_string(&response)?;
                stdout.write_all(response_str.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
            "tools/call" => {
                let tool_name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let arguments = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                match handler.handlers.call_tool(&tool_name, arguments).await {
                    Ok(output) => {
                        let content = serde_json::json!({
                            "type": "text",
                            "text": serde_json::to_string(&output.data).unwrap_or_default(),
                        });
                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": vec![content],
                                "isError": !output.success,
                            },
                        });
                        let response_str = serde_json::to_string(&response)?;
                        stdout.write_all(response_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                    Err(e) => {
                        let response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32603,
                                "message": e.to_string(),
                            },
                        });
                        let response_str = serde_json::to_string(&response)?;
                        stdout.write_all(response_str.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                    }
                }
            }
            _ => {
                if id.is_some() {
                    let response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("Method not found: {method}"),
                        },
                    });
                    let response_str = serde_json::to_string(&response)?;
                    stdout.write_all(response_str.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }
    }

    // Wait for the stdin thread to finish
    let r = stdin_thread.join();
    if r.is_err() {
        tracing::warn!("thread error");
    }

    Ok(())
}
