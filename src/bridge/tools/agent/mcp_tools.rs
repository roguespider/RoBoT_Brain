
// src/tools/agent/mcp_tools.rs
// MCP-specific tool executions

use std::sync::Arc;

use crate::bridge::mcp::McpClient;
use crate::bridge::tools::{get_tools_async, ToolOutput};

use super::inputs::{CallMcpToolInput, ConnectMcpServerInput, GetToolInput};

/// Global MCP client instance
static MCP_CLIENT: std::sync::OnceLock<Arc<McpClient>> = std::sync::OnceLock::new();

/// Initialize the global MCP client
pub fn init_mcp_client(client: Arc<McpClient>) {
    if MCP_CLIENT.set(client).is_err() {
        tracing::warn!("MCP client was already initialized");
    }
}

/// Get the global MCP client
fn get_mcp_client() -> Option<Arc<McpClient>> {
    MCP_CLIENT.get().cloned()
}

/// Execute get_tool tool
pub async fn execute_get_tool(input: GetToolInput) -> Result<ToolOutput, anyhow::Error> {
    let all_tools = get_tools_async().await;

    let tool = all_tools.into_iter().find(|t| t.name == input.name);

    match tool {
        Some(t) => Ok(ToolOutput::success(serde_json::json!({
            "found": true,
            "tool": {
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema
            }
        }))),
        None => Ok(ToolOutput::success(serde_json::json!({
            "found": false,
            "tool": serde_json::Value::Null,
            "error": format!("Tool '{}' not found", input.name)
        }))),
    }
}

/// Execute connect_mcp_server tool - connect to an external MCP server
pub async fn execute_connect_mcp_server(
    input: ConnectMcpServerInput,
) -> Result<ToolOutput, anyhow::Error> {
    let client = match get_mcp_client() {
        Some(c) => c,
        None => {
            return Ok(ToolOutput::success(serde_json::json!({
                "success": false,
                "error": "MCP client not initialized"
            })))
        }
    };

    let args_vec = input.args.unwrap_or_default();
    let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

    match client.connect(&input.name, &input.command, &args).await {
        Ok(()) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "server": input.name,
            "tools": client.list_all_tools().await.len()
        }))),
        Err(e) => Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Execute call_mcp_tool tool - call a tool on a connected MCP server
pub async fn execute_call_mcp_tool(
    input: CallMcpToolInput,
) -> Result<ToolOutput, anyhow::Error> {
    let client = match get_mcp_client() {
        Some(c) => c,
        None => {
            return Ok(ToolOutput::success(serde_json::json!({
                "success": false,
                "error": "MCP client not initialized"
            })))
        }
    };

    // Parse arguments string as JSON if provided
    let arguments = match input.arguments {
        Some(args_str) => match serde_json::from_str(&args_str) {
            Ok(v) => Some(v),
            Err(e) => {
                return Ok(ToolOutput::success(serde_json::json!({
                    "success": false,
                    "error": format!("Invalid JSON in arguments: {}", e)
                })))
            }
        },
        None => None,
    };

    match client.call_tool(&input.tool_name, arguments).await {
        Ok(result) => Ok(ToolOutput::success(serde_json::json!({
            "success": true,
            "result": result
        }))),
        Err(e) => Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}
