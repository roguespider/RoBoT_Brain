// src/bridge/mcp/client/handler.rs
//! Simple MCP client handler implementation

use rmcp::{
    model::ClientInfo,
    ClientHandler,
};

/// A simple MCP client handler
pub struct SimpleClientHandler {
    info: ClientInfo,
}

impl SimpleClientHandler {
    /// Create a new simple client handler
    pub fn new(info: ClientInfo) -> Self {
        Self { info }
    }
}

impl ClientHandler for SimpleClientHandler {
    fn get_info(&self) -> ClientInfo {
        self.info.clone()
    }
}
