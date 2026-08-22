// src/bridge/app/initialization/sub_health_log.rs
//! Log subsystem health diagnostics during startup.

use crate::bridge::app::state::App;

/// Log subsystem health for engines held by App that are otherwise
/// only accessed during construction (Architecture: observability).
pub async fn log_subsystem_health(app: &App) {
    // Hypothesis engine
    let graph_stats = app
        .hypothesis_engine
        .lock()
        .map(|g| g.get_graph_stats())
        .unwrap_or_else(
            |_| crate::experience::hypothesis::support::graph::GraphStats {
                node_count: 0,
                edge_count: 0,
                support_edges: 0,
                contradict_edges: 0,
                depends_edges: 0,
                related_edges: 0,
                cycles: 0,
            },
        );
    tracing::info!(
        "Hypothesis engine ready: {} nodes / {} edges",
        graph_stats.node_count,
        graph_stats.edge_count,
    );

    // Reflection pipeline
    let patterns = app
        .reflection_pipeline
        .analyze_patterns(&[])
        .await
        .unwrap_or_default();
    tracing::info!(
        "Reflection pipeline ready: {} baseline patterns",
        patterns.len(),
    );

    // World model
    let wm_entities = app
        .world_model
        .entities_of_kind(crate::world_model::types::EntityKind::Goal)
        .await;
    tracing::info!(
        "World model ready: {} goal entities tracked",
        wm_entities.len(),
    );

    // Ref-counted subsystems
    tracing::info!(
        "Experience recorder alive: {} strong refs",
        std::sync::Arc::strong_count(&app.experience_recorder),
    );
    tracing::info!(
        "Memory pipeline alive: {} strong refs",
        std::sync::Arc::strong_count(&app.memory_pipeline),
    );
    tracing::info!(
        "Agent loop alive: {} strong refs",
        std::sync::Arc::strong_count(&app.agent_loop),
    );
}
