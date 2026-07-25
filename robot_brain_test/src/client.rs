//! MCP Test Client - connects to server via stdio transport

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout, Command as AsyncCommand};
use tokio::time::timeout;
use serde::{Deserialize, Serialize};

use crate::common::ToolTestResult;

// ============================================================================
// JSON-RPC Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
}

// ============================================================================
// MCP Test Client
// ============================================================================

pub struct McpTestClient {
    child: tokio::process::Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl McpTestClient {
    /// Start the MCP server and connect
    pub async fn start(server_path: PathBuf) -> anyhow::Result<Self> {
        let mut child = AsyncCommand::new(&server_path)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn server {}: {}", server_path.display(), e))?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stdout = BufReader::new(stdout);

        Ok(Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        })
    }

    /// Send a JSON-RPC request and get response with timing
    pub async fn send_request_timed(&mut self, method: &str, params: serde_json::Value) -> anyhow::Result<(JsonRpcResponse, u64)> {
        let start = std::time::Instant::now();
        let response = self.send_request(method, params).await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((response, elapsed))
    }

    /// Send a JSON-RPC request and get response
    async fn send_request(&mut self, method: &str, params: serde_json::Value) -> anyhow::Result<JsonRpcResponse> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        };

        let request_str = serde_json::to_string(&request)?;
        self.stdin.write_all(request_str.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;

        let response_str = self.read_response_line(5).await?;
        
        let response: JsonRpcResponse = serde_json::from_str(&response_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {} - line: {}", e, response_str))?;
        Ok(response)
    }

    async fn read_response_line(&mut self, timeout_secs: u64) -> anyhow::Result<String> {
        let mut line = String::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
        
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(anyhow::anyhow!("Timeout reading response"));
            }
            
            match timeout(remaining, self.stdout.read_line(&mut line)).await {
                Ok(Ok(0)) => return Err(anyhow::anyhow!("EOF reading response")),
                Ok(Ok(_)) => {
                    let trimmed = line.trim();
                    if trimmed.starts_with('{') && trimmed.contains("\"jsonrpc\"") {
                        return Ok(line.clone());
                    }
                    line.clear();
                }
                Ok(Err(e)) => return Err(anyhow::anyhow!("Read error: {}", e)),
                Err(_) => return Err(anyhow::anyhow!("Timeout reading response")),
            }
        }
    }

    /// Initialize connection with server
    pub async fn initialize(&mut self) -> anyhow::Result<serde_json::Value> {
        let response = self.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "clientInfo": {
                "name": "robot_brain_test",
                "version": "1.0.0"
            }
        })).await?;

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        self.stdin.write_all(serde_json::to_string(&notification)?.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;

        response.result
            .ok_or_else(|| anyhow::anyhow!("Initialize failed: {:?}", response.error))
    }

    /// List all available tools
    pub async fn list_tools(&mut self) -> anyhow::Result<Vec<ToolDefinition>> {
        let response = self.send_request("tools/list", serde_json::json!({})).await?;
        
        let result = response.result
            .ok_or_else(|| anyhow::anyhow!("List tools failed: {:?}", response.error))?;
        
        let tools = result.get("tools")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| serde_json::from_value(t.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(tools)
    }

    /// Call a specific tool
    pub async fn call_tool(&mut self, name: &str, arguments: serde_json::Value) -> anyhow::Result<JsonRpcResponse> {
        let response = self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": arguments
        })).await?;
        
        Ok(response)
    }

    /// Call a tool with timing info
    pub async fn call_tool_timed(&mut self, name: &str, arguments: serde_json::Value) -> anyhow::Result<(JsonRpcResponse, u64)> {
        let start = std::time::Instant::now();
        let response = self.call_tool(name, arguments).await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((response, elapsed))
    }

    /// Test a tool and return a ToolTestResult
    pub async fn test_tool(&mut self, name: &str, arguments: serde_json::Value) -> anyhow::Result<ToolTestResult> {
        let start = std::time::Instant::now();
        let result = self.call_tool(name, arguments).await;
        let elapsed = start.elapsed().as_millis() as u64;
        
        match result {
            Ok(response) => {
                let passed = response.error.is_none() && response.result.is_some();
                let message = if passed {
                    format!("Response received in {}ms", elapsed)
                } else if let Some(ref err) = response.error {
                    format!("Error: {}", err.message)
                } else {
                    "No result returned".to_string()
                };
                
                Ok(ToolTestResult {
                    tool_name: name.to_string(),
                    passed,
                    message,
                    response_time_ms: elapsed,
                })
            }
            Err(e) => Ok(ToolTestResult {
                tool_name: name.to_string(),
                passed: false,
                message: format!("Failed to call tool: {}", e),
                response_time_ms: elapsed,
            })
        }
    }

    /// Stop the server gracefully
    pub async fn stop(&mut self) {
        let _ = self.child.kill().await;
    }
}
