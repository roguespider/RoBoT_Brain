// src/bridge/acp/builder.rs
//! ACP message builder

use anyhow::{anyhow, Result};

use super::message::{AcpAgentId, AcpMessage, AcpMessageType};

/// Builder for ACP messages
pub struct AcpMessageBuilder {
    sender: Option<AcpAgentId>,
    receiver: Option<AcpAgentId>,
    message_type: Option<AcpMessageType>,
    payload: Option<serde_json::Value>,
    ttl: Option<u32>,
    conversation_id: Option<String>,
    reply_to: Option<String>,
}

impl AcpMessageBuilder {
    pub fn new() -> Self {
        Self {
            sender: None,
            receiver: None,
            message_type: None,
            payload: None,
            ttl: None,
            conversation_id: None,
            reply_to: None,
        }
    }

    pub fn from(mut self, sender: AcpAgentId) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn to(mut self, receiver: AcpAgentId) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub fn message_type(mut self, message_type: AcpMessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    pub fn payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn ttl(mut self, ttl: u32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn in_conversation(mut self, conversation_id: String) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    pub fn reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    pub fn build(self) -> Result<AcpMessage> {
        let sender = self.sender.ok_or_else(|| anyhow!("sender is required"))?;
        let receiver = self
            .receiver
            .ok_or_else(|| anyhow!("receiver is required"))?;
        let message_type = self
            .message_type
            .ok_or_else(|| anyhow!("message_type is required"))?;
        let payload = self.payload.unwrap_or(serde_json::json!({}));

        let mut msg = AcpMessage::new(sender, receiver, message_type, payload);

        if let Some(ttl) = self.ttl {
            msg.ttl = ttl;
        }
        msg.conversation_id = self.conversation_id;
        msg.reply_to = self.reply_to;

        Ok(msg)
    }
}

impl Default for AcpMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
