// src/api/mcp/handlers/acp_handler/types.rs
//! ACP (Agent Communication Protocol) input types and DynamicAgent

use serde::{Deserialize, Serialize};

use crate::api::acp::{AcpAgent, AcpAgentId, AcpMessage};

/// Input type for list_acp_agents
#[derive(Deserialize, Serialize)]
pub struct ListAcpAgentsInput {}

impl std::fmt::Debug for ListAcpAgentsInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ListAcpAgentsInput").finish()
    }
}

/// Input type for acp_agent_count
#[derive(Deserialize, Serialize)]
pub struct AcpAgentCountInput {}

impl std::fmt::Debug for AcpAgentCountInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpAgentCountInput").finish()
    }
}

/// Input type for acp_router
#[derive(Deserialize, Serialize)]
pub struct GetAcpRouterInput {}

impl std::fmt::Debug for GetAcpRouterInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetAcpRouterInput").finish()
    }
}

/// Input type for acp_registry
#[derive(Deserialize, Serialize)]
pub struct GetAcpRegistryInput {}

impl std::fmt::Debug for GetAcpRegistryInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetAcpRegistryInput").finish()
    }
}

/// Input type for get_agent_capabilities
#[derive(Deserialize, Serialize)]
pub struct GetAgentCapabilitiesInput {
    pub agent_id: String,
}

impl std::fmt::Debug for GetAgentCapabilitiesInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetAgentCapabilitiesInput")
            .field("agent_id", &self.agent_id)
            .finish()
    }
}

/// Input type for get_system_status
#[derive(Deserialize, Serialize)]
pub struct GetSystemStatusInput {}

impl std::fmt::Debug for GetSystemStatusInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetSystemStatusInput").finish()
    }
}

/// Input type for register_agent
#[derive(Deserialize, Serialize)]
pub struct RegisterAgentInput {
    pub agent_type: String,
    pub instance_id: String,
    pub capabilities: Vec<String>,
}

impl std::fmt::Debug for RegisterAgentInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterAgentInput")
            .field("agent_type", &self.agent_type)
            .field("instance_id", &self.instance_id)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

/// Input type for unregister_agent
#[derive(Deserialize, Serialize)]
pub struct UnregisterAgentInput {
    pub agent_type: String,
    pub instance_id: String,
}

impl std::fmt::Debug for UnregisterAgentInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnregisterAgentInput")
            .field("agent_type", &self.agent_type)
            .field("instance_id", &self.instance_id)
            .finish()
    }
}

/// Input type for create_acp_message
#[derive(Deserialize, Serialize)]
pub struct CreateAcpMessageInput {
    pub sender: Option<AcpAgentIdInput>,
    pub receiver: Option<AcpAgentIdInput>,
    pub message_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub ttl: Option<u32>,
    pub reply_to: Option<String>,
    pub conversation_id: Option<String>,
}

impl std::fmt::Debug for CreateAcpMessageInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateAcpMessageInput")
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .field("message_type", &self.message_type)
            .field("payload", &self.payload)
            .field("ttl", &self.ttl)
            .field("reply_to", &self.reply_to)
            .field("conversation_id", &self.conversation_id)
            .finish()
    }
}

/// Input type for route_acp_message
#[derive(Deserialize, Serialize)]
pub struct RouteAcpMessageInput {
    pub sender: Option<AcpAgentIdInput>,
    pub receiver: Option<AcpAgentIdInput>,
    pub message_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub ttl: Option<u32>,
    pub reply_to: Option<String>,
    pub conversation_id: Option<String>,
}

impl std::fmt::Debug for RouteAcpMessageInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouteAcpMessageInput")
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .field("message_type", &self.message_type)
            .field("payload", &self.payload)
            .field("ttl", &self.ttl)
            .field("reply_to", &self.reply_to)
            .field("conversation_id", &self.conversation_id)
            .finish()
    }
}

/// Input type for AcpAgentId (used in sender/receiver)
#[derive(Deserialize, Serialize)]
pub struct AcpAgentIdInput {
    pub agent_type: String,
    pub instance_id: String,
}

impl std::fmt::Debug for AcpAgentIdInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpAgentIdInput")
            .field("agent_type", &self.agent_type)
            .field("instance_id", &self.instance_id)
            .finish()
    }
}

/// Dynamic ACP agent implementation for runtime registration
pub struct DynamicAgent {
    id: AcpAgentId,
    capabilities: Vec<String>,
}

impl DynamicAgent {
    pub fn new(agent_type: &str, instance_id: &str, capabilities: Vec<String>) -> Self {
        Self {
            id: AcpAgentId::new(agent_type, instance_id),
            capabilities,
        }
    }
}

impl AcpAgent for DynamicAgent {
    fn id(&self) -> &AcpAgentId {
        &self.id
    }

    fn handle(&self, message: AcpMessage) -> anyhow::Result<Option<AcpMessage>> {
        let response_payload = serde_json::json!({
            "status": "handled",
            "agent": self.id.uri(),
            "original_action": message.payload.get("action"),
            "capabilities": self.capabilities,
        });
        Ok(Some(message.reply(response_payload)))
    }
}
