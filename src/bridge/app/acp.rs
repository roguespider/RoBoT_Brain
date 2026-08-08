// src/bridge/app/acp.rs
//! ACP (Agent Communication Protocol) methods for the App

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::acp::{AcpRegistry, AcpMessage, AcpRouter, AcpAgentId};

use super::state::App;

/// Get reference to ACP router
pub fn acp_router(app: &App) -> Arc<AcpRouter> {
    app.acp_router.clone()
}

/// Get ACP registry for agent registration
pub fn acp_registry(app: &App) -> Arc<AcpRegistry> {
    app.acp_router.registry()
}

/// Route an ACP message to the appropriate agent
pub fn route_acp_message(app: &App, message: AcpMessage) -> Result<Option<AcpMessage>> {
    app.acp_router.route(message)
}

/// List all registered ACP agents
pub fn list_acp_agents(app: &App) -> Result<Vec<AcpAgentId>> {
    app.acp_router.registry().list_agents()
}

/// Get count of registered ACP agents
pub fn acp_agent_count(app: &App) -> usize {
    app.acp_router.registry().count()
}
