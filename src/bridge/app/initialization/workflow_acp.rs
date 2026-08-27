// src/bridge/app/initialization/workflow_acp.rs
//! Wire planner, create workflow engine, and set up ACP router/registry.

use std::sync::Arc;

use crate::bridge::acp::{AcpRegistry, AcpRouter};
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::metrics::MetricsCollector;
use crate::memory::retrieval::MemoryRetrieval;
use crate::planner::{Planner, PolicyEngine};
use crate::workflows::engine::WorkflowEngine;

/// Build the planner, workflow engine, and ACP router/registry.
///
/// Wires personality creativity and policy engine into the planner,
/// creates the workflow engine, then creates and configures the ACP
/// router/registry with handlers and system agents.
pub fn setup_planner_workflow_acp(
    metrics: &Arc<MetricsCollector>,
    database: &Arc<crate::database::sqlite::SqliteDatabase>,
    coordinator: &Arc<ExperienceCoordinator>,
    policy_engine: &Arc<PolicyEngine>,
    shared_personality: &Arc<std::sync::Mutex<crate::personality::Personality>>,
    memory_retrieval: Option<Arc<MemoryRetrieval>>,
) -> (
    Arc<Planner>,
    Arc<WorkflowEngine>,
    Arc<AcpRouter>,
    Arc<AcpRegistry>,
) {
    // Create planning system
    let mut planner = Planner::new(metrics.clone());

    // Wire personality creativity into planner for decision-making
    let shared_personality_clone = shared_personality.clone();
    planner.set_creativity_check(
        move |complexity: f32| match shared_personality_clone.lock() {
            Ok(guard) => guard.should_use_creativity(complexity),
            Err(poisoned) => {
                tracing::error!("Personality mutex poisoned in creativity check");
                poisoned.into_inner().should_use_creativity(complexity)
            }
        },
    );

    // Wire policy engine into planner for policy-based action gating
    planner.set_policy_engine(policy_engine.clone());
    let planner = Arc::new(planner);

    // Create workflow engine with database access, coordinator for event integration,
    // and memory retrieval for automatic context enrichment
    let workflow_engine = Arc::new(WorkflowEngine::with_database_and_coordinator(
        metrics.clone(),
        database.clone(),
        coordinator.clone(),
        memory_retrieval,
    ));
    tracing::info!("Workflow engine initialized with coordinator and memory retrieval");

    // Create ACP router and registry
    let acp_registry = Arc::new(AcpRegistry::new());
    let acp_router = Arc::new(AcpRouter::new(acp_registry.clone()));

    // Register a default Inform broadcast handler so broadcast-style ACP
    // messages are observed even when no agent-specific handler exists.
    acp_router
        .register_handler(crate::bridge::acp::message::AcpMessageType::Inform, |msg| {
            tracing::info!(
                "ACP Inform broadcast received from {}: {}",
                msg.sender,
                msg.payload,
            );
            Ok(None)
        })
        .map_err(|e| anyhow::anyhow!("Failed to register ACP Inform handler: {}", e))
        .ok();

    // Register system agents
    let system_agent = crate::bridge::acp::system_agent::create_system_agent();
    let worker_agent = crate::bridge::acp::system_agent::create_worker_agent();
    let _ = acp_registry.register(system_agent);
    let _ = acp_registry.register(worker_agent);
    tracing::info!("ACP system agents registered (system:main, worker:1)");

    (planner, workflow_engine, acp_router, acp_registry)
}
