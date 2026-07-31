// src/bridge/mcp/client.rs

//! MCP Client for connecting to external MCP servers
//!
//! This module provides the client-side implementation for connecting to
//! external MCP (Model Context Protocol) servers via child process transport.
//!
//! # Example
//!
//! ```rust,ignore
//! use robot_brain::bridge::mcp::McpClient;
//!
//! let client = McpClient::new();
//! client.connect("my-server", "npx", &["-y", "@server/plugin"]).await?;
//! let tools = client.list_all_tools().await;
//! let result = client.call_tool("my_tool", Some(json!({"arg": "value"}))).await?;
//! ```

use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation, Tool},
    service::{Peer, RoleClient, RunningService},
    ClientHandler,
};
use tokio::process::Command;
use tokio::sync::RwLock;

/// A connected MCP server
struct ConnectedServer {
    name: String,
    /// The running service - kept alive to maintain the connection
    running: RunningService<RoleClient, SimpleClientHandler>,
    /// Cached tools from this server
    tools: Vec<Tool>,
}

impl ConnectedServer {
    /// Get peer for making requests
    #[allow(dead_code)]
    fn peer(&self) -> Peer<RoleClient> {
        self.running.peer().clone()
    }
}

/// Tool invocation error
#[derive(Debug, Clone)]
pub struct ToolError {
    pub message: String,
    pub server: String,
    pub tool: String,
}

#[allow(dead_code)]
impl ToolError {
    /// Create a new tool error
    pub fn new(server: &str, tool: &str, message: &str) -> Self {
        Self {
            message: message.to_string(),
            server: server.to_string(),
            tool: tool.to_string(),
        }
    }

    /// Create an error for tool not found
    pub fn not_found(tool: &str) -> Self {
        Self {
            message: format!("Tool '{}' not found on any connected server", tool),
            server: "unknown".to_string(),
            tool: tool.to_string(),
        }
    }

    /// Create an error for connection failure
    pub fn connection_failed(server: &str, error: &str) -> Self {
        Self {
            message: format!("Failed to connect to server '{}': {}", server, error),
            server: server.to_string(),
            tool: String::new(),
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.server, self.tool, self.message)
    }
}

impl std::error::Error for ToolError {}

/// MCP Client for connecting to external MCP servers
pub struct McpClient {
    /// Connected servers and their tools
    servers: Arc<RwLock<Vec<ConnectedServer>>>,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if client has any connected servers
    #[allow(dead_code)]
    pub async fn has_connections(&self) -> bool {
        !self.servers.read().await.is_empty()
    }

    /// Get number of connected servers
    #[allow(dead_code)]
    pub async fn server_count(&self) -> usize {
        self.servers.read().await.len()
    }

    /// Connect to an MCP server via child process transport
    pub async fn connect(&self, name: &str, command: &str, args: &[&str]) -> Result<()> {
        use rmcp::transport::child_process::TokioChildProcess;

        tracing::info!(
            "Connecting to MCP server '{}': {} {:?}",
            name,
            command,
            args
        );

        // Create child process transport
        let mut cmd = Command::new(command);
        cmd.args(args);
        let transport = TokioChildProcess::new(cmd)?;

        // Create client handler
        let client = SimpleClientHandler {
            info: ClientInfo::new(
                ClientCapabilities::default(),
                Implementation::new("robot_brain", env!("CARGO_PKG_VERSION")),
            ),
        };

        // Start the client and get the running service
        let running = rmcp::serve_client(client, transport).await?;

        // Get the peer to list tools
        let peer = running.peer().clone();

        // List tools from the server
        let tools = match peer.list_all_tools().await {
            Ok(tools) => {
                tracing::info!("Server '{}' exposed {} tools", name, tools.len());
                tools
            }
            Err(e) => {
                tracing::warn!("Failed to list tools from '{}': {}", name, e);
                Vec::new()
            }
        };

        let tools_count = tools.len();

        // Store the server connection
        let server = ConnectedServer {
            name: name.to_string(),
            running,
            tools,
        };

        self.servers.write().await.push(server);

        tracing::info!(
            "MCP client connected to '{}' with {} tools",
            name,
            tools_count
        );
        Ok(())
    }

    /// Disconnect from a server by name
    #[allow(dead_code)]
    pub async fn disconnect(&self, name: &str) -> Result<bool> {
        let mut servers = self.servers.write().await;
        if let Some(pos) = servers.iter().position(|s| s.name == name) {
            servers.remove(pos);
            tracing::info!("Disconnected from MCP server '{}'", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disconnect from all servers
    #[allow(dead_code)]
    pub async fn disconnect_all(&self) -> usize {
        let mut servers = self.servers.write().await;
        let count = servers.len();
        servers.clear();
        tracing::info!("Disconnected from {} MCP servers", count);
        count
    }

    /// List all servers
    #[allow(dead_code)]
    pub async fn list_servers(&self) -> Vec<String> {
        self.servers.read().await.iter().map(|s| s.name.clone()).collect()
    }

    /// List tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<Tool> {
        let servers = self.servers.read().await;
        let mut tools = Vec::new();
        for server in servers.iter() {
            tools.extend(server.tools.clone());
        }
        tools
    }

    /// Get a specific tool by name
    #[allow(dead_code)]
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let servers = self.servers.read().await;
        for server in servers.iter() {
            if let Some(tool) = server.tools.iter().find(|t| t.name == name) {
                return Some(tool.clone());
            }
        }
        None
    }

    /// Get the server that owns a specific tool
    #[allow(dead_code)]
    pub async fn get_tool_server(&self, tool_name: &str) -> Option<String> {
        let servers = self.servers.read().await;
        for server in servers.iter() {
            if server.tools.iter().any(|t| t.name == tool_name) {
                return Some(server.name.clone());
            }
        }
        None
    }

    /// Refresh tools from a specific server
    #[allow(dead_code)]
    pub async fn refresh_tools(&self, server_name: &str) -> Result<Vec<Tool>> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.iter_mut().find(|s| s.name == server_name) {
            let peer = server.running.peer().clone();
            let tools = peer.list_all_tools().await?;
            server.tools = tools.clone();
            tracing::info!("Refreshed {} tools from server '{}'", tools.len(), server_name);
            Ok(tools)
        } else {
            Err(anyhow::anyhow!("Server '{}' not found", server_name))
        }
    }

    /// Call a tool on a connected server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ToolError> {
        // Find the server that has this tool
        let (server_name, peer) = {
            let servers = self.servers.read().await;
            let found = servers
                .iter()
                .find(|s| s.tools.iter().any(|t| t.name == tool_name))
                .ok_or_else(|| ToolError::not_found(tool_name))?;
            (found.name.clone(), found.running.peer().clone())
        };

        // Call the tool via the server's peer
        let params = match arguments {
            Some(v) => CallToolRequestParams::new(tool_name.to_string())
                .with_arguments(v.as_object().cloned().unwrap_or_default()),
            None => CallToolRequestParams::new(tool_name.to_string()),
        };

        match peer.call_tool(params).await {
            Ok(result) => {
                // Extract content from the result
                if let Some(content) = result.content.first() {
                    if let Some(text) = content.as_text() {
                        match serde_json::from_str(&text.text) {
                            Ok(json) => Ok(json),
                            Err(_) => Ok(serde_json::json!(text.text)),
                        }
                    } else {
                        Ok(serde_json::json!(content))
                    }
                } else {
                    Ok(serde_json::json!({"result": "ok"}))
                }
            }
            Err(e) => Err(ToolError::new(&server_name, tool_name, &format!("Tool call failed: {:?}", e))),
        }
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple MCP client handler
struct SimpleClientHandler {
    info: ClientInfo,
}

impl ClientHandler for SimpleClientHandler {
    fn get_info(&self) -> ClientInfo {
        self.info.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = McpClient::new();
        assert!(!client.has_connections().await);
        assert_eq!(client.server_count().await, 0);
    }

    #[tokio::test]
    async fn test_list_servers_empty() {
        let client = McpClient::new();
        let servers = client.list_servers().await;
        assert!(servers.is_empty());
    }

    #[tokio::test]
    async fn test_list_tools_empty() {
        let client = McpClient::new();
        let tools = client.list_all_tools().await;
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_get_tool_not_found() {
        let client = McpClient::new();
        let tool = client.get_tool("nonexistent").await;
        assert!(tool.is_none());
    }

    #[tokio::test]
    async fn test_get_tool_server_not_found() {
        let client = McpClient::new();
        let server = client.get_tool_server("nonexistent").await;
        assert!(server.is_none());
    }

    #[test]
    fn test_tool_error_display() {
        let error = ToolError::new("server1", "tool1", "Something went wrong");
        assert_eq!(error.to_string(), "[server1] tool1: Something went wrong");
    }

    #[test]
    fn test_tool_error_not_found() {
        let error = ToolError::not_found("my_tool");
        assert!(error.message.contains("my_tool"));
        assert_eq!(error.server, "unknown");
        assert_eq!(error.tool, "my_tool");
    }

    #[test]
    fn test_tool_error_connection_failed() {
        let error = ToolError::connection_failed("server1", "connection refused");
        assert!(error.message.contains("server1"));
        assert!(error.message.contains("connection refused"));
        assert_eq!(error.server, "server1");
    }
}
