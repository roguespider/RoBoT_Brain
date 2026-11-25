// src/api/mcp/handlers/acp_handler/mod.rs
//! ACP (Agent Communication Protocol) tools handler - directory module

pub mod handler;
pub mod types;

// Re-export public types for backward compatibility
pub use types::{
    AcpAgentCountInput, AcpAgentIdInput, CreateAcpMessageInput, DynamicAgent, GetAcpRegistryInput,
    GetAcpRouterInput, GetAgentCapabilitiesInput, GetSystemStatusInput, ListAcpAgentsInput,
    RegisterAgentInput, RouteAcpMessageInput, UnregisterAgentInput,
};

// Re-export the handler
pub use handler::AcpToolsHandler;
