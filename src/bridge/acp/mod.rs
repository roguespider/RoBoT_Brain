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
pub mod message;
pub mod registry;
pub mod router;
pub mod system_agent;

// Re-export production types only
pub use agent::AcpAgent;
pub use message::{AcpAgentId, AcpMessage};
pub use registry::AcpRegistry;
pub use router::AcpRouter;
