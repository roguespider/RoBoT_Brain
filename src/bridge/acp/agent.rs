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

#[cfg(test)]
pub(crate) mod test_types {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Agent capability description
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AcpCapability {
        pub name: String,
        pub description: String,
        pub input_schema: serde_json::Value,
        pub output_schema: serde_json::Value,
    }

    impl AcpCapability {
        pub fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
            }
        }
    }

    /// Simple agent implementation with handler closure
    pub struct SimpleAgent {
        id: AcpAgentId,
        description: String,
        capabilities: Vec<AcpCapability>,
        handler: Box<dyn Fn(AcpMessage) -> Result<Option<AcpMessage>> + Send + Sync>,
    }

    impl SimpleAgent {
        pub fn new(
            id: AcpAgentId,
            description: &str,
            capabilities: Vec<AcpCapability>,
            handler: impl Fn(AcpMessage) -> Result<Option<AcpMessage>> + Send + Sync + 'static,
        ) -> Self {
            Self {
                id,
                description: description.to_string(),
                capabilities,
                handler: Box::new(handler),
            }
        }

        pub fn description(&self) -> &str {
            &self.description
        }

        pub fn capabilities(&self) -> &[AcpCapability] {
            &self.capabilities
        }
    }

    impl AcpAgent for SimpleAgent {
        fn id(&self) -> &AcpAgentId {
            &self.id
        }

        fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>> {
            (self.handler)(message)
        }
    }
}
