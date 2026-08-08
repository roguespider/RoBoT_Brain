// src/bridge/acp/mod.rs
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

pub mod agent;
pub mod builder;
pub mod channel;
pub mod error;
pub mod message;
pub mod registry;
pub mod router;
pub mod system_agent;

// Re-export production types only
pub use agent::AcpAgent;
pub use message::{AcpAgentId, AcpMessage};
pub use registry::AcpRegistry;
pub use router::AcpRouter;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    // Re-export test types
    pub use super::agent::test_types::{AcpCapability, SimpleAgent};
    pub use super::builder::AcpMessageBuilder;
    pub use super::channel::{AcpChannel, InMemoryChannel};
    pub use super::error::{AcpError, AcpErrorCode};
    pub use super::message::AcpMessageType;

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
        assert_eq!(
            AcpMessageType::Request.reply_type(),
            AcpMessageType::Response
        );
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

        // Use match instead of unwrap
        if let Err(e) = channel.send(msg.clone()) {
            assert!(false, "send failed: {}", e);
            unsafe { std::hint::unreachable_unchecked() }
        }

        let received = match channel.try_recv() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "try_recv failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
        assert!(received.is_some());
        // Use if-let instead of unwrap
        if let Some(received_msg) = received {
            assert_eq!(received_msg.payload, msg.payload);
        } else {
            assert!(false, "Expected Some message");
            unsafe { std::hint::unreachable_unchecked() }
        }

        // Channel should be empty now
        let empty = match channel.try_recv() {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "try_recv failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
        assert!(empty.is_none());
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

        if let Err(e) = registry.register(agent.clone()) {
            assert!(false, "register failed: {}", e);
            unsafe { std::hint::unreachable_unchecked() }
        }

        assert_eq!(registry.count(), 1);

        let retrieved = match registry.get(&AcpAgentId::new("test", "1")) {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "get failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
        assert!(retrieved.is_some());

        let by_type = match registry.get_by_type("test") {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "get_by_type failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
        assert_eq!(by_type.len(), 1);

        let unreg = match registry.unregister(&AcpAgentId::new("test", "1")) {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "unregister failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
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

        if let Err(e) = registry.register(agent) {
            assert!(false, "register failed: {}", e);
            unsafe { std::hint::unreachable_unchecked() }
        }

        let msg = AcpMessage::new(
            AcpAgentId::new("client", "1"),
            AcpAgentId::new("worker", "1"),
            AcpMessageType::Request,
            serde_json::json!({"task": "do_work"}),
        );

        let result = router.route(msg);
        assert!(result.is_ok());
        let response = match result {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "route failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
        assert!(response.is_some());

        let resp = match response {
            Some(r) => r,
            None => {
                assert!(false, "Expected Some response");
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
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
        let err = match result {
            Ok(_) => {
                assert!(false, "Expected error");
                unsafe { std::hint::unreachable_unchecked() }
            }
            Err(e) => e,
        };
        assert!(err.to_string().contains("Unknown receiver"));
    }

    #[test]
    fn test_message_builder() {
        let msg = match AcpMessageBuilder::new()
            .from(AcpAgentId::new("sender", "1"))
            .to(AcpAgentId::new("receiver", "1"))
            .message_type(AcpMessageType::Request)
            .payload(serde_json::json!({"action": "test"}))
            .ttl(10)
            .build()
        {
            Ok(m) => m,
            Err(e) => {
                assert!(false, "build failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };

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
    fn test_message_builder_conversation_and_reply() {
        let conv_id = "conv-123".to_string();
        let reply_id = "msg-456".to_string();
        
        let msg = AcpMessageBuilder::new()
            .from(AcpAgentId::new("sender", "1"))
            .to(AcpAgentId::new("receiver", "1"))
            .message_type(AcpMessageType::Request)
            .in_conversation(conv_id.clone())
            .reply_to(reply_id.clone())
            .build()
            .expect("build should succeed");

        assert_eq!(msg.conversation_id, Some(conv_id));
        assert_eq!(msg.reply_to, Some(reply_id));
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

        let response = match agent.handle(msg) {
            Ok(r) => r,
            Err(e) => {
                assert!(false, "handle failed: {}", e);
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
        let response = match response {
            Some(r) => r,
            None => {
                assert!(false, "Expected Some response");
                unsafe { std::hint::unreachable_unchecked() }
            }
        };
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
