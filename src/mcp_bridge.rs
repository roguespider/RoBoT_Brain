// src/mcp_bridge.rs

use anyhow::Result;
use std::sync::Arc;

use crate::experience::coordinator::ExperienceCoordinator;

/// MCP communication layer.
///
/// This will eventually:
/// - expose tools to MCP clients
/// - receive requests from LM Studio / Zed
/// - route requests into the experience system
/// - return responses
///
/// For now it is the runtime shell.
pub struct McpBridge {
    coordinator: Arc<ExperienceCoordinator>,
}

impl McpBridge {
    /// Create MCP bridge.
    pub async fn new(coordinator: Arc<ExperienceCoordinator>) -> Result<Self> {
        Ok(Self { coordinator })
    }

    /// Start MCP runtime loop.
    ///
    /// The real MCP transport loop will live here.
    pub async fn run(self) -> Result<()> {
        println!("MCP bridge running");

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    }
}
