// src/bridge/app/initialization/acp_diagnostics.rs
//! ACP routing health check (explicit diagnostics, P2-001C).

use crate::bridge::app::state::App;

/// Send a diagnostic query through the ACP router to verify message routing.
pub fn run_acp_health_check(app: &App) {
    use crate::bridge::app::{
        acp_agent_count, acp_registry, acp_router, list_acp_agents, route_acp_message,
    };

    let router = acp_router(app);
    let registry = acp_registry(app);
    let agent_count = acp_agent_count(app);
    tracing::info!(
        "ACP subsystem: router_ready={} registry_agents={} {} agent(s) registered",
        !router
            .registry()
            .list_agents()
            .unwrap_or_default()
            .is_empty()
            || agent_count == 0,
        registry.count(),
        agent_count
    );
    let agents = match list_acp_agents(app) {
        Ok(agents) => agents,
        Err(e) => {
            tracing::warn!("ACP diagnostics: failed to list agents: {}", e);
            return;
        }
    };
    for agent_id in &agents {
        tracing::info!("Registered ACP agent: {}", agent_id);
    }

    // Diagnostic: count agents by type so the registry's type-indexed
    // lookup is exercised.
    match router.registry().get_by_type("worker") {
        Ok(worker_agents) => {
            tracing::info!("ACP worker agents by type: {}", worker_agents.len());
        }
        Err(e) => {
            tracing::warn!("ACP diagnostics: failed to query agents by type: {}", e);
        }
    }

    // Send a diagnostic query to the system agent to verify message routing
    let system_id = crate::bridge::acp::AcpAgentId::new("system", "main");
    let diagnostic_msg = crate::bridge::acp::AcpMessage::new(
        system_id.clone(),
        system_id,
        crate::bridge::acp::message::AcpMessageType::Query,
        serde_json::json!({"query": "diagnostics_health_check"}),
    );
    match route_acp_message(app, diagnostic_msg) {
        Ok(Some(reply)) => {
            tracing::info!(
                "ACP diagnostics health check: received reply of type {:?}",
                reply.message_type
            );
        }
        Ok(None) => {
            tracing::info!("ACP diagnostics health check: message routed (no reply)");
        }
        Err(e) => tracing::warn!("ACP diagnostics health check failed: {}", e),
    }
}
