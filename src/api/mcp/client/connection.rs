// src/bridge/mcp/client/connection.rs
//! Connected server management

use rmcp::{
    model::Tool,
    service::{Peer, RoleClient, RunningService},
};

use super::handler::SimpleClientHandler;

/// A connected MCP server
pub struct ConnectedServer {
    name: String,
    /// The running service - kept alive to maintain the connection
    pub(crate) running: RunningService<RoleClient, SimpleClientHandler>,
    /// Cached tools from this server
    pub(crate) tools: Vec<Tool>,
}

impl ConnectedServer {
    /// Create a new connected server
    pub fn new(name: String, running: RunningService<RoleClient, SimpleClientHandler>, tools: Vec<Tool>) -> Self {
        Self {
            name,
            running,
            tools,
        }
    }

    /// Get the server name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the tools
    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// Get peer for making requests
    pub fn peer(&self) -> Peer<RoleClient> {
        self.running.peer().clone()
    }

    /// Update the cached tools
    pub fn update_tools(&mut self, tools: Vec<Tool>) {
        self.tools = tools;
    }
}
