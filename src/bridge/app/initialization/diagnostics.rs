// src/bridge/app/initialization/diagnostics.rs
//! Explicit subsystem diagnostics (P2-001A/B/C).
//!
//! Production startup (`App::new` / `App::run`) initializes production
//! systems only. All subsystem self-tests / lifecycle probes live here and
//! run exclusively when the user explicitly requests diagnostics via the
//! `robot diagnose` CLI command. This preserves existing test coverage
//! (P2-001B) while removing probe pollution from production startup (P2-001A).
//!
//! Production startup (`App::new` / `App::run`) initializes production
//! systems only. All subsystem self-tests / lifecycle probes live here and
//! run exclusively when the user explicitly requests diagnostics via the
//! `robot diagnose` CLI command.

use crate::bridge::app::state::App;

/// Run the full subsystem diagnostic suite and log a summary.
///
/// Exercises the lifecycle of every subsystem that previously ran probes at
/// startup, so those code paths stay live without polluting production
/// startup (P2-001A/P2-001C).
pub async fn run_startup_diagnostics(app: &App) {
    tracing::info!("Starting explicit subsystem diagnostics");

    // Learning subsystem lifecycle probes
    crate::bridge::app::initialization::candidates::verify_candidates().await;
    crate::bridge::app::initialization::working_memory::verify_working_memory().await;
    crate::bridge::app::initialization::lineage_tracker::verify_lineage_tracker().await;
    crate::bridge::app::initialization::hypothesis_manager::verify_hypothesis_manager().await;
    crate::bridge::app::initialization::learning_pipeline::verify_learning_pipeline().await;
    crate::bridge::app::initialization::exploration_repo::verify_exploration_repository().await;

    // Experience repository persistence probe (uses the live database)
    let database = app.mcp_context.database.clone();
    crate::bridge::app::initialization::experience_repo::verify_experience_repository(&database)
        .await;

    // JobQueue durability probe (uses the live queue + database)
    let job_queue = app.mcp_context.job_queue.clone();
    crate::bridge::app::initialization::job_queue::verify_job_queue(&job_queue, &database);

    // Metrics subsystem self-check
    let metrics_summary = crate::experience::metrics::run_metrics_self_check().await;
    tracing::info!("{}", metrics_summary);

    // Personality subsystem self-check
    crate::bridge::app::initialization::personality_diagnostics::run_personality_self_check(app);

    // MCP client connection-management probe
    crate::bridge::app::initialization::mcp_client_diagnostics::run_mcp_client_probe(app).await;

    // ACP routing health check
    crate::bridge::app::initialization::acp_diagnostics::run_acp_health_check(app);

    // Policy engine management probe
    let policy_engine = app.mcp_context.policy.clone();
    crate::bridge::app::initialization::policy::verify_policy_management(&policy_engine).await;

    // Worker manager enqueue/completion probes
    let worker_manager = app.mcp_context.worker_manager.clone();
    crate::bridge::app::initialization::worker_diagnostics::run_worker_probes(&worker_manager)
        .await;

    // Scheduler task-management probe
    let scheduler = app.mcp_context.scheduler.clone();
    crate::bridge::app::initialization::scheduler_diagnostics::run_scheduler_probe(&scheduler)
        .await;

    // Learning pipeline construction-path probes
    let metrics = app.mcp_context.metrics.clone();
    let reflection_engine = app.mcp_context.reflection.clone();
    let evolution_engine = app.mcp_context.evolution.clone();
    crate::bridge::app::initialization::learning_diagnostics::run_learning_probes(
        &metrics,
        &reflection_engine,
        &evolution_engine,
    )
    .await;

    // ExperienceRecorder convenience helper verification
    crate::bridge::app::initialization::experience_recorder_diagnostics::verify_experience_recorder(
        app,
    );

    // Reputation system verification
    crate::bridge::app::initialization::reputation_diagnostics::verify_reputation_system(app);

    // Reflection/hypothesis type-surface verification (P2-001B)
    crate::bridge::app::initialization::reflection_surface_diagnostics::verify_type_surfaces(app)
        .await;

    // ReflectionPipeline and ReflectionEngine verification
    crate::bridge::app::initialization::reflection_diagnostics::verify_reflection_pipeline(app)
        .await;
    crate::bridge::app::initialization::reflection_diagnostics::verify_reflection_engine(app).await;

    // HypothesisPipeline verification
    crate::bridge::app::initialization::hypothesis_pipeline_diagnostics::verify_hypothesis_pipeline(app).await;

    // Subsystem health logging
    crate::bridge::app::initialization::sub_health_log::log_subsystem_health(app).await;

    tracing::info!("Subsystem diagnostics complete");
}
