// src/bridge/acp/agent.rs
//! ACP agent implementations

use anyhow::Result;

use super::message::{AcpAgentId, AcpMessage};

/// ACP agent trait - defines the interface for ACP agents
pub trait AcpAgent: Send + Sync {
    /// Get the agent's ID
    fn id(&self) -> &AcpAgentId;

    /// Get the agent's capability descriptions
    fn capabilities(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Handle an incoming ACP message
    fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>>;
}

