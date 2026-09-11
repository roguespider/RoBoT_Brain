// src/bridge/acp/message.rs
//! ACP message types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ACP message envelope
#[derive(Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    pub id: String,
    pub sender: AcpAgentId,
    pub receiver: AcpAgentId,
    pub message_type: AcpMessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub conversation_id: Option<String>,
    pub reply_to: Option<String>,
    pub ttl: u32,
}

impl std::fmt::Debug for AcpMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpMessage")
            .field("id", &self.id)
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .field("message_type", &self.message_type)
            .field("payload", &self.payload)
            .field("timestamp", &self.timestamp)
            .field("conversation_id", &self.conversation_id)
            .field("reply_to", &self.reply_to)
            .field("ttl", &self.ttl)
            .finish()
    }
}

impl AcpMessage {
    /// Create a new ACP message
    pub fn new(
        sender: AcpAgentId,
        receiver: AcpAgentId,
        message_type: AcpMessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            receiver,
            message_type,
            payload,
            timestamp: Utc::now(),
            conversation_id: None,
            reply_to: None,
            ttl: 64, // Default TTL
        }
    }

    /// Create a reply to this message
    pub fn reply(&self, payload: serde_json::Value) -> AcpMessage {
        let mut reply = Self::new(
            self.receiver.clone(),
            self.sender.clone(),
            self.message_type.reply_type(),
            payload,
        );
        reply.conversation_id = self
            .conversation_id
            .clone()
            .or_else(|| Some(self.id.clone()));
        reply.reply_to = Some(self.id.clone());
        reply
    }
}

/// Agent identifier
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcpAgentId {
    pub agent_type: String,
    pub instance_id: String,
}

impl std::fmt::Debug for AcpAgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcpAgentId")
            .field("agent_type", &self.agent_type)
            .field("instance_id", &self.instance_id)
            .finish()
    }
}

impl AcpAgentId {
    /// Create a new agent ID
    pub fn new(agent_type: &str, instance_id: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            instance_id: instance_id.to_string(),
        }
    }

    /// Get the full agent URI
    pub fn uri(&self) -> String {
        format!("acp://{}/{}", self.agent_type, self.instance_id)
    }
}

impl std::fmt::Display for AcpAgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.agent_type, self.instance_id)
    }
}

/// ACP message types
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpMessageType {
    /// Request: asking the receiver to perform an action
    Request,
    /// Response: responding to a request
    Response,
    /// Query: asking for information
    Query,
    /// Inform: informing the receiver of something
    Inform,
    /// Acknowledge: acknowledging receipt of a message
    Ack,
    /// Error: reporting an error
    Error,
    /// Subscribe: subscribe to updates
    Subscribe,
    /// Unsubscribe: unsubscribe from updates
    Unsubscribe,
    /// Publish: publish an event
    Publish,
}

impl std::fmt::Debug for AcpMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request => write!(f, "AcpMessageType::Request"),
            Self::Response => write!(f, "AcpMessageType::Response"),
            Self::Query => write!(f, "AcpMessageType::Query"),
            Self::Inform => write!(f, "AcpMessageType::Inform"),
            Self::Ack => write!(f, "AcpMessageType::Ack"),
            Self::Error => write!(f, "AcpMessageType::Error"),
            Self::Subscribe => write!(f, "AcpMessageType::Subscribe"),
            Self::Unsubscribe => write!(f, "AcpMessageType::Unsubscribe"),
            Self::Publish => write!(f, "AcpMessageType::Publish"),
        }
    }
}

impl AcpMessageType {
    /// Get the reply message type for this message type
    pub fn reply_type(&self) -> Self {
        match self {
            Self::Request => Self::Response,
            Self::Query => Self::Response,
            Self::Inform => Self::Ack,
            Self::Subscribe => Self::Ack,
            Self::Unsubscribe => Self::Ack,
            _ => Self::Inform,
        }
    }

    /// Check if this message type expects a reply
    pub fn expects_reply(&self) -> bool {
        matches!(
            self,
            Self::Request | Self::Query | Self::Subscribe | Self::Unsubscribe
        )
    }
}
