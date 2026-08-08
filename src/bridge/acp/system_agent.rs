// src/bridge/acp/system_agent.rs
//! System agent implementation for handling ACP messages

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::agent::AcpAgent;
use super::message::{AcpAgentId, AcpMessage, AcpMessageType};

/// System agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapability {
    pub name: String,
    pub description: String,
    pub version: String,
}

/// System agent for handling ACP messages
pub struct SystemAgent {
    id: AcpAgentId,
    capabilities: Vec<SystemCapability>,
}

impl SystemAgent {
    pub fn new() -> Self {
        let id = AcpAgentId::new("system", "main");
        
        let capabilities = vec![
            SystemCapability {
                name: "message_handling".to_string(),
                description: "Handles all incoming ACP messages".to_string(),
                version: "1.0".to_string(),
            },
            SystemCapability {
                name: "query".to_string(),
                description: "Responds to query messages".to_string(),
                version: "1.0".to_string(),
            },
            SystemCapability {
                name: "broadcast".to_string(),
                description: "Handles broadcast messages".to_string(),
                version: "1.0".to_string(),
            },
        ];
        
        Self { id, capabilities }
    }
    
    pub fn agent_id(&self) -> &AcpAgentId {
        &self.id
    }
    
    pub fn get_capabilities(&self) -> &[SystemCapability] {
        &self.capabilities
    }
}

impl Default for SystemAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl AcpAgent for SystemAgent {
    fn id(&self) -> &AcpAgentId {
        &self.id
    }

    fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>> {
        // Create a response message
        let response_payload = serde_json::json!({
            "status": "received",
            "original_action": message.payload.get("action"),
            "message_type": format!("{:?}", message.message_type),
        });
        
        let response = message.reply(response_payload);
        Ok(Some(response))
    }
}

/// Worker agent for handling worker messages
pub struct WorkerAgent {
    id: AcpAgentId,
    capabilities: Vec<SystemCapability>,
}

impl WorkerAgent {
    pub fn new() -> Self {
        let id = AcpAgentId::new("worker", "1");
        
        let capabilities = vec![
            SystemCapability {
                name: "task_processing".to_string(),
                description: "Processes task requests".to_string(),
                version: "1.0".to_string(),
            },
        ];
        
        Self { id, capabilities }
    }
    
    pub fn agent_id(&self) -> &AcpAgentId {
        &self.id
    }
    
    pub fn get_capabilities(&self) -> &[SystemCapability] {
        &self.capabilities
    }
}

impl Default for WorkerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl AcpAgent for WorkerAgent {
    fn id(&self) -> &AcpAgentId {
        &self.id
    }

    fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>> {
        // Create a response message
        let response_payload = serde_json::json!({
            "status": "processed",
            "original_action": message.payload.get("action"),
            "message_type": format!("{:?}", message.message_type),
            "worker": "worker:1",
        });
        
        let response = message.reply(response_payload);
        Ok(Some(response))
    }
}

/// Create a boxed system agent
pub fn create_system_agent() -> Arc<SystemAgent> {
    Arc::new(SystemAgent::new())
}

/// Create a boxed worker agent
pub fn create_worker_agent() -> Arc<WorkerAgent> {
    Arc::new(WorkerAgent::new())
}
