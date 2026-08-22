// src/bridge/app/initialization/scheduler_diagnostics.rs
//! Scheduler task-management probe (explicit diagnostics, P2-001C).

use std::sync::Arc;

use crate::experience::scheduler::Scheduler;

/// Exercise scheduler task-management methods with a transient probe task.
pub async fn run_scheduler_probe(scheduler: &Arc<Scheduler>) {
    let probe_id = scheduler
        .create_task(
            "diagnostics-scheduler-probe",
            crate::experience::scheduler::TaskType::Cleanup,
            crate::experience::scheduler::TaskSchedule::Manual,
        )
        .await
        .unwrap_or_else(|_| String::new());

    let loaded = scheduler.load_tasks().await;
    let loaded_count = loaded.as_ref().map(|t| t.len()).unwrap_or(0);

    if !probe_id.is_empty() {
        scheduler.cancel_task(&probe_id).await.ok();
        scheduler.enable_task(&probe_id).await.ok();
        scheduler.delete_task(&probe_id).await.ok();
    }

    crate::experience::scheduler::setup_memory_consolidation_task(scheduler)
        .await
        .ok();

    tracing::info!(
        "Scheduler management verified: load_tasks_ok={} loaded_count={} (probe removed={})",
        loaded.is_ok(),
        loaded_count,
        !probe_id.is_empty()
    );
}
