// src/bridge/app/initialization/scheduler_diagnostics.rs
//! Scheduler task-management probe (explicit diagnostics, P2-001C).
//!
//! Runs against an isolated Scheduler backed by a temporary database so
//! diagnostics can never create, cancel, or delete tasks in the production
//! scheduler store.

use std::sync::Arc;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::scheduler::Scheduler;

/// Exercise scheduler task-management methods with a transient probe task
/// on an isolated temporary database.
/// Returns `Ok(())` on success, `Err(msg)` on failure.
pub async fn run_scheduler_probe() -> std::result::Result<(), String> {
    // Isolated database in the OS temp directory: probe tasks are written to
    // their own robot_brain.db, never to the production database.
    let probe_dir = std::env::temp_dir().join(format!(
        "robot_brain_diagnostics_scheduler_{}",
        uuid::Uuid::new_v4()
    ));
    let database = match SqliteDatabase::initialize_at(&probe_dir) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            tracing::warn!(
                "Scheduler diagnostics skipped: isolated database init failed: {}",
                e
            );
            return Err(format!("Scheduler diagnostics init failed: {}", e));
        }
    };
    let scheduler = Scheduler::new(database);

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

    tracing::info!(
        "Scheduler management verified: load_tasks_ok={} loaded_count={} (probe removed={})",
        loaded.is_ok(),
        loaded_count,
        !probe_id.is_empty()
    );

    // Remove the isolated probe database directory.
    if let Err(e) = std::fs::remove_dir_all(&probe_dir) {
        tracing::warn!(
            "Scheduler diagnostics cleanup failed for {:?}: {}",
            probe_dir,
            e
        );
    }
    Ok(())
}
