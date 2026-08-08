// src/bridge/acp/router.rs
//! ACP router for message routing

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use super::message::{AcpMessage, AcpMessageType};
use super::registry::AcpRegistry;

/// ACP router for routing messages between agents
pub struct AcpRouter {
    registry: Arc<AcpRegistry>,
    handlers: std::sync::RwLock<
        HashMap<String, Box<dyn Fn(AcpMessage) -> Result<Option<AcpMessage>> + Send + Sync>>,
    >,
}

impl AcpRouter {
    pub fn new(registry: Arc<AcpRegistry>) -> Self {
        Self {
            registry,
            handlers: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Route a message to the appropriate agent
    pub fn route(&self, message: AcpMessage) -> Result<Option<AcpMessage>> {
        // Check for custom handlers first
        let type_name = format!("{:?}", message.message_type);
        if let Ok(handlers) = self.handlers.read() {
            if let Some(handler) = handlers.get(&type_name) {
                return handler(message);
            }
        }

        // Route to registered agent
        let agent = self.registry.get(&message.receiver)?;

        match agent {
            Some(agent) => agent.handle(message),
            None => Err(anyhow!("Unknown receiver: {}", message.receiver)),
        }
    }

    /// Register a custom message handler for a message type
    pub fn register_handler(
        &self,
        message_type: AcpMessageType,
        handler: impl Fn(AcpMessage) -> Result<Option<AcpMessage>> + Send + Sync + 'static,
    ) -> Result<()> {
        let type_name = format!("{:?}", message_type);
        let mut handlers = self
            .handlers
            .write()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
        handlers.insert(type_name, Box::new(handler));
        Ok(())
    }

    /// Get the registry
    pub fn registry(&self) -> Arc<AcpRegistry> {
        Arc::clone(&self.registry)
    }
}
