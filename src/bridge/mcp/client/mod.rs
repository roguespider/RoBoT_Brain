// src/bridge/mcp/client/mod.rs
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

pub mod connection;
pub mod error;
pub mod handler;

pub use error::ToolError;

use std::sync::Arc;

use anyhow::Result;
use rmcp::model::{CallToolRequestParams, ClientCapabilities, ClientInfo, Implementation, Tool};
use tokio::process::Command;
use tokio::sync::RwLock;

use connection::ConnectedServer;
use handler::SimpleClientHandler;

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
    pub async fn has_connections(&self) -> bool {
        !self.servers.read().await.is_empty()
    }

    /// Get number of connected servers
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
        let client = SimpleClientHandler::new(ClientInfo::new(
            ClientCapabilities::default(),
            Implementation::new("robot_brain", env!("CARGO_PKG_VERSION")),
        ));

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
        let server = ConnectedServer::new(name.to_string(), running, tools);

        self.servers.write().await.push(server);

        tracing::info!(
            "MCP client connected to '{}' with {} tools",
            name,
            tools_count
        );
        Ok(())
    }

    /// Disconnect from a server by name
    pub async fn disconnect(&self, name: &str) -> Result<bool> {
        let mut servers = self.servers.write().await;
        if let Some(pos) = servers.iter().position(|s| s.name() == name) {
            servers.remove(pos);
            tracing::info!("Disconnected from MCP server '{}'", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disconnect from all servers
    pub async fn disconnect_all(&self) -> usize {
        let mut servers = self.servers.write().await;
        let count = servers.len();
        servers.clear();
        tracing::info!("Disconnected from {} MCP servers", count);
        count
    }

    /// List all servers
    pub async fn list_servers(&self) -> Vec<String> {
        self.servers.read().await.iter().map(|s| s.name().to_string()).collect()
    }

    /// List tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<Tool> {
        let servers = self.servers.read().await;
        let mut tools = Vec::new();
        for server in servers.iter() {
            tools.extend(server.tools().to_vec());
        }
        tools
    }

    /// Get a specific tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let servers = self.servers.read().await;
        for server in servers.iter() {
            if let Some(tool) = server.tools().iter().find(|t| t.name == name) {
                return Some(tool.clone());
            }
        }
        None
    }

    /// Get the server that owns a specific tool
    pub async fn get_tool_server(&self, tool_name: &str) -> Option<String> {
        let servers = self.servers.read().await;
        for server in servers.iter() {
            if server.tools().iter().any(|t| t.name == tool_name) {
                return Some(server.name().to_string());
            }
        }
        None
    }

    /// Refresh tools from a specific server
    pub async fn refresh_tools(&self, server_name: &str) -> Result<Vec<Tool>> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.iter_mut().find(|s| s.name() == server_name) {
            let peer = server.peer();
            let tools = peer.list_all_tools().await?;
            let tools_clone = tools.clone();
            server.update_tools(tools);
            tracing::info!("Refreshed {} tools from server '{}'", tools_clone.len(), server_name);
            Ok(tools_clone)
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
                .find(|s| s.tools().iter().any(|t| t.name == tool_name))
                .ok_or_else(|| ToolError::not_found(tool_name))?;
            (found.name().to_string(), found.peer())
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
