// src/bridge/acp.rs

//! ACP (Agent Communication Protocol) for inter-agent communication
//!
//! Provides a message-based protocol for agents to communicate, request actions,
//! query information, and coordinate activities.
//!
//! # Example
//!
//! ```rust
//! use robot_brain::bridge::acp::{
//!     AcpRouter, AcpRegistry, AcpMessage, AcpAgentId, AcpMessageType,
//! };
//!
//! // Create router and registry
//! let registry = Arc::new(AcpRegistry::new());
//! let router = AcpRouter::new(registry.clone());
//!
//! // Route a message
//! let msg = AcpMessage::new(
//!     AcpAgentId::new("client", "1"),
//!     AcpAgentId::new("server", "1"),
//!     AcpMessageType::Request,
//!     serde_json::json!({"action": "get_status"}),
//! );
//! router.route(msg)?;
//! ```

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

// ============================================================================
// Message Types
// ============================================================================

/// ACP message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Create a new message with custom TTL
    pub fn with_ttl(
        sender: AcpAgentId,
        receiver: AcpAgentId,
        message_type: AcpMessageType,
        payload: serde_json::Value,
        ttl: u32,
    ) -> Self {
        let mut msg = Self::new(sender, receiver, message_type, payload);
        msg.ttl = ttl;
        msg
    }

    /// Create a reply to this message
    pub fn reply(&self, payload: serde_json::Value) -> AcpMessage {
        let mut reply = Self::new(
            self.receiver.clone(),
            self.sender.clone(),
            self.message_type.reply_type(),
            payload,
        );
        reply.conversation_id = self.conversation_id.clone().or_else(|| Some(self.id.clone()));
        reply.reply_to = Some(self.id.clone());
        reply
    }

    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        self.ttl == 0
    }

    /// Decrement TTL and return whether message is still valid
    pub fn decrement_ttl(&mut self) -> bool {
        if self.ttl > 0 {
            self.ttl -= 1;
        }
        !self.is_expired()
    }

    /// Forward this message to a new receiver
    pub fn forward_to(&self, new_receiver: AcpAgentId) -> AcpMessage {
        let mut forwarded = Self::new(
            self.sender.clone(),
            new_receiver,
            self.message_type.clone(),
            self.payload.clone(),
        );
        forwarded.conversation_id = self.conversation_id.clone();
        forwarded.reply_to = Some(self.id.clone());
        forwarded
    }
}

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcpAgentId {
    pub agent_type: String,
    pub instance_id: String,
}

impl AcpAgentId {
    /// Create a new agent ID
    pub fn new(agent_type: &str, instance_id: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            instance_id: instance_id.to_string(),
        }
    }

    /// Create a new agent ID with a random instance
    pub fn with_random_instance(agent_type: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            instance_id: Uuid::new_v4().to_string()[..8].to_string(),
        }
    }

    /// Get the full agent URI
    pub fn uri(&self) -> String {
        format!("acp://{}/{}", self.agent_type, self.instance_id)
    }

    /// Get broadcast address for this agent type
    pub fn broadcast(agent_type: &str) -> Self {
        Self {
            agent_type: agent_type.to_string(),
            instance_id: "*".to_string(),
        }
    }

    /// Check if this is a broadcast ID
    pub fn is_broadcast(&self) -> bool {
        self.instance_id == "*"
    }
}

impl std::fmt::Display for AcpAgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.agent_type, self.instance_id)
    }
}

// ============================================================================
// Message Types Enum
// ============================================================================

/// ACP message types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        matches!(self, Self::Request | Self::Query | Self::Subscribe)
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// ACP protocol errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpError {
    pub code: AcpErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AcpError {
    pub fn new(code: AcpErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl std::fmt::Display for AcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AcpError {}

/// ACP error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcpErrorCode {
    MalformedMessage,
    UnknownReceiver,
    NotAuthorized,
    NotFound,
    InvalidPayload,
    Timeout,
    InternalError,
}

impl AcpErrorCode {
    pub fn to_code(&self) -> u16 {
        match self {
            Self::MalformedMessage => 1001,
            Self::UnknownReceiver => 1002,
            Self::NotAuthorized => 1003,
            Self::NotFound => 1004,
            Self::InvalidPayload => 1005,
            Self::Timeout => 1006,
            Self::InternalError => 1999,
        }
    }
}

// ============================================================================
// Channel Trait & Implementation
// ============================================================================

/// ACP channel for sending and receiving messages
pub trait AcpChannel: Send + Sync {
    /// Send a message through the channel
    fn send(&self, message: AcpMessage) -> Result<()>;

    /// Receive a message from the channel (non-blocking)
    fn try_recv(&self) -> Result<Option<AcpMessage>>;

    /// Get channel name for debugging
    fn name(&self) -> &str;
}

/// In-memory channel for local agent communication
pub struct InMemoryChannel {
    name: String,
    messages: Arc<std::sync::Mutex<Vec<AcpMessage>>>,
    waiting: Arc<AtomicBool>,
}

impl InMemoryChannel {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            messages: Arc::new(std::sync::Mutex::new(Vec::new())),
            waiting: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_buffer(name: &str, capacity: usize) -> Self {
        Self {
            name: name.to_string(),
            messages: Arc::new(std::sync::Mutex::new(Vec::with_capacity(capacity))),
            waiting: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AcpChannel for InMemoryChannel {
    fn send(&self, message: AcpMessage) -> Result<()> {
        let mut messages = self.messages.lock().map_err(|_| anyhow!("Lock poisoned"))?;
        messages.push(message);
        self.waiting.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn try_recv(&self) -> Result<Option<AcpMessage>> {
        let mut messages = self.messages.lock().map_err(|_| anyhow!("Lock poisoned"))?;
        self.waiting.store(false, Ordering::SeqCst);
        Ok(messages.pop())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// Agent Trait & Implementation
// ============================================================================

/// ACP agent trait
pub trait AcpAgent: Send + Sync {
    /// Get the agent's ID
    fn id(&self) -> &AcpAgentId;

    /// Handle an incoming ACP message
    fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>>;

    /// Get the agent's capabilities
    fn capabilities(&self) -> Vec<AcpCapability>;

    /// Get agent description
    fn description(&self) -> &str;
}

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
}

impl AcpAgent for SimpleAgent {
    fn id(&self) -> &AcpAgentId {
        &self.id
    }

    fn handle(&self, message: AcpMessage) -> Result<Option<AcpMessage>> {
        (self.handler)(message)
    }

    fn capabilities(&self) -> Vec<AcpCapability> {
        self.capabilities.clone()
    }

    fn description(&self) -> &str {
        &self.description
    }
}

// ============================================================================
// Registry
// ============================================================================

/// ACP registry for agent discovery
pub struct AcpRegistry {
    agents: std::sync::RwLock<HashMap<AcpAgentId, Arc<dyn AcpAgent>>>,
}

impl AcpRegistry {
    pub fn new() -> Self {
        Self {
            agents: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Register an agent
    pub fn register(&self, agent: Arc<dyn AcpAgent>) -> Result<()> {
        let id = agent.id().clone();
        let mut agents = self
            .agents
            .write()
            .map_err(|_| anyhow!("Lock poisoned"))?;
        agents.insert(id, agent);
        Ok(())
    }

    /// Unregister an agent
    pub fn unregister(&self, id: &AcpAgentId) -> Result<Option<Arc<dyn AcpAgent>>> {
        let mut agents = self
            .agents
            .write()
            .map_err(|_| anyhow!("Lock poisoned"))?;
        Ok(agents.remove(id))
    }

    /// Get an agent by ID
    pub fn get(&self, id: &AcpAgentId) -> Result<Option<Arc<dyn AcpAgent>>> {
        let agents = self
            .agents
            .read()
            .map_err(|_| anyhow!("Lock poisoned"))?;
        Ok(agents.get(id).cloned())
    }

    /// Get all agents of a specific type
    pub fn get_by_type(&self, agent_type: &str) -> Result<Vec<Arc<dyn AcpAgent>>> {
        let agents = self
            .agents
            .read()
            .map_err(|_| anyhow!("Lock poisoned"))?;
        Ok(agents
            .values()
            .filter(|a| a.id().agent_type == agent_type)
            .cloned()
            .collect())
    }

    /// List all registered agent IDs
    pub fn list_agents(&self) -> Result<Vec<AcpAgentId>> {
        let agents = self
            .agents
            .read()
            .map_err(|_| anyhow!("Lock poisoned"))?;
        Ok(agents.keys().cloned().collect())
    }

    /// Count registered agents
    pub fn count(&self) -> usize {
        self.agents.read().map(|g| g.len()).unwrap_or(0)
    }
}

impl Default for AcpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Router
// ============================================================================

/// ACP router for routing messages between agents
pub struct AcpRouter {
    registry: Arc<AcpRegistry>,
    handlers: std::sync::RwLock<HashMap<String, Box<dyn Fn(AcpMessage) -> Result<Option<AcpMessage>> + Send + Sync>>>,
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
            .map_err(|_| anyhow!("Lock poisoned"))?;
        handlers.insert(type_name, Box::new(handler));
        Ok(())
    }

    /// Get the registry
    pub fn registry(&self) -> Arc<AcpRegistry> {
        Arc::clone(&self.registry)
    }
}

// ============================================================================
// Message Builder
// ============================================================================

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
        let receiver = self.receiver.ok_or_else(|| anyhow!("receiver is required"))?;
        let message_type = self.message_type.ok_or_else(|| anyhow!("message_type is required"))?;
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id() {
        let id = AcpAgentId::new("test", "123");
        assert_eq!(id.agent_type, "test");
        assert_eq!(id.instance_id, "123");
        assert_eq!(id.uri(), "acp://test/123");
    }

    #[test]
    fn test_agent_id_broadcast() {
        let id = AcpAgentId::broadcast("workers");
        assert!(id.is_broadcast());
        assert_eq!(id.uri(), "acp://workers/*");
    }

    #[test]
    fn test_message_creation() {
        let msg = AcpMessage::new(
            AcpAgentId::new("sender", "1"),
            AcpAgentId::new("receiver", "2"),
            AcpMessageType::Request,
            serde_json::json!({"action": "test"}),
        );
        
        assert!(!msg.id.is_empty());
        assert_eq!(msg.conversation_id, None);
        assert_eq!(msg.reply_to, None);
        assert_eq!(msg.ttl, 64);
    }

    #[test]
    fn test_message_reply() {
        let original = AcpMessage::new(
            AcpAgentId::new("sender", "1"),
            AcpAgentId::new("receiver", "2"),
            AcpMessageType::Request,
            serde_json::json!({"action": "test"}),
        );
        
        let reply = original.reply(serde_json::json!({"status": "ok"}));
        
        assert_eq!(reply.sender, original.receiver);
        assert_eq!(reply.receiver, original.sender);
        assert_eq!(reply.message_type, AcpMessageType::Response);
        assert_eq!(reply.reply_to, Some(original.id.clone()));
        assert_eq!(reply.conversation_id, Some(original.id.clone()));
    }

    #[test]
    fn test_message_ttl() {
        let mut msg = AcpMessage::new(
            AcpAgentId::new("a", "1"),
            AcpAgentId::new("b", "1"),
            AcpMessageType::Inform,
            serde_json::json!({}),
        );
        
        assert!(!msg.is_expired());
        assert!(msg.decrement_ttl());
        assert_eq!(msg.ttl, 63);
        
        // Decrement to 0 - returns false when message becomes expired
        msg.ttl = 1;
        assert!(!msg.decrement_ttl()); // Returns false when message expires
        assert!(msg.is_expired());
    }

    #[test]
    fn test_message_type_reply() {
        assert_eq!(AcpMessageType::Request.reply_type(), AcpMessageType::Response);
        assert_eq!(AcpMessageType::Query.reply_type(), AcpMessageType::Response);
        assert_eq!(AcpMessageType::Inform.reply_type(), AcpMessageType::Ack);
        assert_eq!(AcpMessageType::Error.reply_type(), AcpMessageType::Inform);
    }

    #[test]
    fn test_message_type_expects_reply() {
        assert!(AcpMessageType::Request.expects_reply());
        assert!(AcpMessageType::Query.expects_reply());
        assert!(AcpMessageType::Subscribe.expects_reply());
        assert!(!AcpMessageType::Response.expects_reply());
        assert!(!AcpMessageType::Ack.expects_reply());
    }

    #[test]
    fn test_in_memory_channel() {
        let channel = InMemoryChannel::new("test_channel");
        
        let msg = AcpMessage::new(
            AcpAgentId::new("a", "1"),
            AcpAgentId::new("b", "1"),
            AcpMessageType::Request,
            serde_json::json!({"test": true}),
        );
        
        channel.send(msg.clone()).unwrap();
        
        let received = channel.try_recv().unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().payload, msg.payload);
        
        // Channel should be empty now
        assert!(channel.try_recv().unwrap().is_none());
    }

    #[test]
    fn test_registry() {
        let registry = AcpRegistry::new();
        
        let agent = Arc::new(SimpleAgent::new(
            AcpAgentId::new("test", "1"),
            "Test agent",
            vec![AcpCapability::new("test_cap", "Test capability")],
            |msg| Ok(Some(msg.reply(serde_json::json!({"handled": true})))),
        ));
        
        registry.register(agent.clone()).unwrap();
        
        assert_eq!(registry.count(), 1);
        
        let retrieved = registry.get(&AcpAgentId::new("test", "1")).unwrap();
        assert!(retrieved.is_some());
        
        let by_type = registry.get_by_type("test").unwrap();
        assert_eq!(by_type.len(), 1);
        
        let unreg = registry.unregister(&AcpAgentId::new("test", "1")).unwrap();
        assert!(unreg.is_some());
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_router() {
        let registry = Arc::new(AcpRegistry::new());
        let router = AcpRouter::new(registry.clone());
        
        let agent = Arc::new(SimpleAgent::new(
            AcpAgentId::new("worker", "1"),
            "Worker agent",
            vec![],
            |msg| {
                let reply = msg.reply(serde_json::json!({"processed": true}));
                Ok(Some(reply))
            },
        ));
        
        registry.register(agent).unwrap();
        
        let msg = AcpMessage::new(
            AcpAgentId::new("client", "1"),
            AcpAgentId::new("worker", "1"),
            AcpMessageType::Request,
            serde_json::json!({"task": "do_work"}),
        );
        
        let result = router.route(msg);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_some());
        
        let resp = response.unwrap();
        assert_eq!(resp.message_type, AcpMessageType::Response);
        assert!(resp.payload.get("processed").is_some());
    }

    #[test]
    fn test_router_unknown_receiver() {
        let registry = Arc::new(AcpRegistry::new());
        let router = AcpRouter::new(registry);
        
        let msg = AcpMessage::new(
            AcpAgentId::new("a", "1"),
            AcpAgentId::new("unknown", "1"),
            AcpMessageType::Request,
            serde_json::json!({}),
        );
        
        let result = router.route(msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown receiver"));
    }

    #[test]
    fn test_message_builder() {
        let msg = AcpMessageBuilder::new()
            .from(AcpAgentId::new("sender", "1"))
            .to(AcpAgentId::new("receiver", "1"))
            .message_type(AcpMessageType::Request)
            .payload(serde_json::json!({"action": "test"}))
            .ttl(10)
            .build()
            .unwrap();
        
        assert_eq!(msg.sender.agent_type, "sender");
        assert_eq!(msg.receiver.agent_type, "receiver");
        assert_eq!(msg.message_type, AcpMessageType::Request);
        assert_eq!(msg.ttl, 10);
    }

    #[test]
    fn test_message_builder_missing_fields() {
        let result = AcpMessageBuilder::new()
            .from(AcpAgentId::new("a", "1"))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_simple_agent() {
        let agent = SimpleAgent::new(
            AcpAgentId::new("echo", "1"),
            "Echo agent",
            vec![AcpCapability::new("echo", "Echoes messages")],
            |msg| Ok(Some(msg.reply(serde_json::json!({"echo": msg.payload})))),
        );
        
        assert_eq!(agent.id().agent_type, "echo");
        assert_eq!(agent.description(), "Echo agent");
        assert_eq!(agent.capabilities().len(), 1);
        
        let msg = AcpMessage::new(
            AcpAgentId::new("client", "1"),
            AcpAgentId::new("echo", "1"),
            AcpMessageType::Query,
            serde_json::json!({"ping": true}),
        );
        
        let response = agent.handle(msg).unwrap().unwrap();
        assert_eq!(response.message_type, AcpMessageType::Response);
        assert_eq!(response.payload["echo"]["ping"], true);
    }

    #[test]
    fn test_error() {
        let error = AcpError::new(AcpErrorCode::NotFound, "Resource not found")
            .with_details(serde_json::json!({"resource": "test_id"}));
        
        assert_eq!(error.code, AcpErrorCode::NotFound);
        assert_eq!(error.message, "Resource not found");
        assert!(error.details.is_some());
        assert_eq!(error.code.to_code(), 1004);
    }
}
