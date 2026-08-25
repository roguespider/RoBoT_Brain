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

/// Aggregate result from running all subsystem diagnostics.
pub struct DiagnosticResult {
    pub passed: usize,
    pub failed: usize,
}

/// Per-subsystem diagnostic outcome.
use crate::bridge::app::initialization;
use crate::bridge::app::state::App;
use crate::experience::metrics;

/// Run the full subsystem diagnostic suite and log a per-subsystem summary.
///
/// Exercises the lifecycle of every subsystem that previously ran probes at
/// startup, so those code paths stay live without polluting production
/// startup (P2-001A/P2-001C).
///
/// Returns a `DiagnosticResult` with pass/fail counts so the CLI handler
/// can exit with code 1 when any diagnostic fails.
pub async fn run_startup_diagnostics(app: &App) -> DiagnosticResult {
    tracing::info!("Starting explicit subsystem diagnostics");
    let mut passed = 0usize;
    let mut failed = 0usize;
    // Track per-subsystem results for the summary output (P2-001C-M4).
    let mut results: Vec<(&str, bool)> = Vec::new();

    // Learning subsystem lifecycle probes
    let ok = initialization::candidates::verify_candidates().await;
    let ok = ok.is_ok();
    if ok {
        passed += 1;
    } else {
        failed += 1;
    }
    results.push(("candidates", ok));

    let ok = initialization::working_memory::verify_working_memory().await;
    let ok = ok.is_ok();
    if ok {
        passed += 1;
    } else {
        failed += 1;
    }
    results.push(("working_memory", ok));

    let ok = initialization::lineage_tracker::verify_lineage_tracker().await;
    let ok = ok.is_ok();
    if ok {
        passed += 1;
    } else {
        failed += 1;
    }
    results.push(("lineage_tracker", ok));

    let ok = initialization::hypothesis_manager::verify_hypothesis_manager().await;
    let ok = ok.is_ok();
    if ok {
        passed += 1;
    } else {
        failed += 1;
    }
    results.push(("hypothesis_manager", ok));

    let ok = initialization::learning_pipeline::verify_learning_pipeline().await;
    let ok = ok.is_ok();
    if ok {
        passed += 1;
    } else {
        failed += 1;
    }
    results.push(("learning_pipeline", ok));

    let ok = initialization::exploration_repo::verify_exploration_repository().await;
    let ok = ok.is_ok();
    if ok {
        passed += 1;
    } else {
        failed += 1;
    }
    results.push(("exploration_repo", ok));

    // Experience repository persistence probe (isolated temporary
    // database; the production database is never touched)
    let exp_repo_ok = initialization::experience_repo::verify_experience_repository().await;
    if exp_repo_ok {
        tracing::info!("Experience repository verified");
        passed += 1;
    } else {
        tracing::error!("Experience repository verification failed");
        failed += 1;
    }
    results.push(("experience_repo", exp_repo_ok));

    // JobQueue durability probe (isolated temporary database; the live
    // queue and production database are never touched)
    let jq_ok = initialization::job_queue::verify_job_queue();
    if jq_ok {
        tracing::info!("JobQueue verified");
        passed += 1;
    } else {
        tracing::error!("JobQueue verification failed");
        failed += 1;
    }
    results.push(("job_queue", jq_ok));

    // Metrics subsystem self-check
    let metrics_result = metrics::run_metrics_self_check().await;
    let metrics_ok = metrics_result.is_ok();
    if metrics_ok {
        tracing::info!("Metrics self-check passed");
        passed += 1;
    } else {
        if let Err(ref e) = metrics_result {
            tracing::error!("Metrics self-check failed: {e}");
        }
        failed += 1;
    }
    results.push(("metrics", metrics_ok));

    // Personality subsystem self-check
    let personality_result =
        initialization::personality_diagnostics::run_personality_self_check(app);
    let personality_ok = personality_result.is_ok();
    if personality_ok {
        tracing::info!("Personality self-check passed");
        passed += 1;
    } else {
        if let Err(ref e) = personality_result {
            tracing::error!("Personality self-check failed: {e}");
        }
        failed += 1;
    }
    results.push(("personality", personality_ok));

    // MCP client connection-management probe
    let mcp_result = initialization::mcp_client_diagnostics::run_mcp_client_probe(app).await;
    let mcp_ok = mcp_result.is_ok();
    if mcp_ok {
        tracing::info!("MCP client probe passed");
        passed += 1;
    } else {
        if let Err(ref e) = mcp_result {
            tracing::error!("MCP client probe failed: {e}");
        }
        failed += 1;
    }
    results.push(("mcp_client", mcp_ok));

    // ACP routing health check
    let acp_result = initialization::acp_diagnostics::run_acp_health_check(app);
    let acp_ok = acp_result.is_ok();
    if acp_ok {
        tracing::info!("ACP health check passed");
        passed += 1;
    } else {
        if let Err(ref e) = acp_result {
            tracing::error!("ACP health check failed: {e}");
        }
        failed += 1;
    }
    results.push(("acp", acp_ok));

    // Policy engine management probe
    let policy_engine = app.mcp_context.policy.clone();
    let policy_ok = initialization::policy::verify_policy_management(&policy_engine)
        .await
        .is_ok();
    if policy_ok {
        tracing::info!("Policy management verified");
        passed += 1;
    } else {
        tracing::error!("Policy management verification failed");
        failed += 1;
    }
    results.push(("policy", policy_ok));

    // Worker manager enqueue/completion probes (isolated bus + temp
    // database queue; the live bus and production queue are never touched)
    let worker_result = initialization::worker_diagnostics::run_worker_probes().await;
    let worker_ok = worker_result.is_ok();
    if worker_ok {
        tracing::info!("Worker probes passed");
        passed += 1;
    } else {
        if let Err(ref e) = worker_result {
            tracing::error!("Worker probes failed: {e}");
        }
        failed += 1;
    }
    results.push(("worker", worker_ok));

    // Scheduler task-management probe (isolated temporary database; the
    // production scheduler store is never touched)
    let scheduler_ok = initialization::scheduler_diagnostics::run_scheduler_probe()
        .await
        .is_ok();
    if scheduler_ok {
        tracing::info!("Scheduler probe passed");
        passed += 1;
    } else {
        tracing::error!("Scheduler probe failed");
        failed += 1;
    }
    results.push(("scheduler", scheduler_ok));

    // Learning pipeline construction-path probes
    let metrics = app.mcp_context.metrics.clone();
    let reflection_engine = app.mcp_context.reflection.clone();
    let evolution_engine = app.mcp_context.evolution.clone();
    let learning_ok = initialization::learning_diagnostics::run_learning_probes(
        &metrics,
        &reflection_engine,
        &evolution_engine,
    )
    .await
    .is_ok();
    if learning_ok {
        tracing::info!("Learning probes passed");
        passed += 1;
    } else {
        tracing::error!("Learning probes failed");
        failed += 1;
    }
    results.push(("learning", learning_ok));

    // ExperienceRecorder convenience helper verification (isolated temp DB).
    let recorder_ok =
        initialization::experience_recorder_diagnostics::verify_experience_recorder().is_ok();
    if recorder_ok {
        tracing::info!("ExperienceRecorder verification passed");
        passed += 1;
    } else {
        tracing::error!("ExperienceRecorder verification failed");
        failed += 1;
    }
    results.push(("experience_recorder", recorder_ok));

    // Reputation system verification
    let reputation_ok =
        initialization::reputation_diagnostics::verify_reputation_system(app).is_ok();
    if reputation_ok {
        tracing::info!("Reputation system verification passed");
        passed += 1;
    } else {
        tracing::error!("Reputation system verification failed");
        failed += 1;
    }
    results.push(("reputation", reputation_ok));

    // Reflection/hypothesis type-surface verification (P2-001B)
    let surfaces_ok = initialization::reflection_surface_diagnostics::verify_type_surfaces(app)
        .await
        .is_ok();
    if surfaces_ok {
        tracing::info!("Type surfaces verification passed");
        passed += 1;
    } else {
        tracing::error!("Type surfaces verification failed");
        failed += 1;
    }
    results.push(("reflection_surfaces", surfaces_ok));

    // ReflectionPipeline and ReflectionEngine verification
    let rp_ok = initialization::reflection_diagnostics::verify_reflection_pipeline(app)
        .await
        .is_ok();
    if rp_ok {
        tracing::info!("ReflectionPipeline verification passed");
        passed += 1;
    } else {
        tracing::error!("ReflectionPipeline verification failed");
        failed += 1;
    }
    results.push(("reflection_pipeline", rp_ok));

    let re_ok = initialization::reflection_diagnostics::verify_reflection_engine(app)
        .await
        .is_ok();
    if re_ok {
        tracing::info!("ReflectionEngine verification passed");
        passed += 1;
    } else {
        tracing::error!("ReflectionEngine verification failed");
        failed += 1;
    }
    results.push(("reflection_engine", re_ok));

    // HypothesisPipeline verification
    let hp_ok = initialization::hypothesis_pipeline_diagnostics::verify_hypothesis_pipeline(app)
        .await
        .is_ok();
    if hp_ok {
        tracing::info!("HypothesisPipeline verification passed");
        passed += 1;
    } else {
        tracing::error!("HypothesisPipeline verification failed");
        failed += 1;
    }
    results.push(("hypothesis_pipeline", hp_ok));

    // Subsystem health logging
    initialization::sub_health_log::log_subsystem_health(app).await;

    let result = DiagnosticResult { passed, failed };

    // Log per-subsystem summary (P2-001C-M4)
    let mut summary = String::new();
    for (name, ok) in &results {
        summary.push(' ');
        summary.push_str(name);
        summary.push_str(": ");
        summary.push_str(if *ok { "[PASS]" } else { "[FAIL]" });
    }
    tracing::info!(
        "Subsystem diagnostics complete: passed={} failed={}\n  subsystems:{}",
        result.passed,
        result.failed,
        summary
    );
    result
}
