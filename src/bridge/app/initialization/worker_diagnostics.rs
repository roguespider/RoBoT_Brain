// src/bridge/app/initialization/worker_diagnostics.rs
//! Worker manager probes (explicit diagnostics, P2-001C).
//!
//! Runs against an isolated WorkerManager (own bus + own temporary-database
//! job queue) so diagnostics can never enqueue onto the live bus or mutate
//! the production queue.

use std::sync::{Arc, Mutex};

use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::queue::JobQueue;
use crate::experience::worker_manager::WorkerManager;

/// Exercise the targeted enqueue path and the manager-level completion path
/// on an isolated worker manager.
///
/// Returns `Err` if the isolated database cannot be initialized.
pub async fn run_worker_probes() -> std::result::Result<(), String> {
    // Isolated database in the OS temp directory: probe jobs are written to
    // their own robot_brain.db, never to the production database.
    let probe_dir = std::env::temp_dir().join(format!(
        "robot_brain_diagnostics_worker_{}",
        uuid::Uuid::new_v4()
    ));
    let database = SqliteDatabase::initialize_at(&probe_dir).map_err(|e| {
        format!(
            "Worker manager diagnostics: isolated database init failed: {}",
            e
        )
    })?;
    let database = Arc::new(database);
    let isolated_bus = Arc::new(ExperienceBus::new());
    let isolated_queue = Arc::new(Mutex::new(JobQueue::with_database(database)));
    let worker_manager = Arc::new(WorkerManager::new_with_queue(isolated_bus, isolated_queue));

    // Verify the targeted enqueue path (single-observer dispatch with a unique
    // job ID per P0-002) so WorkerManager::enqueue stays live alongside the
    // broadcast path. The probe event is accepted by every observer; workers
    // mark it complete via their callbacks.
    let probe_event = crate::experience::events::ExperienceEvent::recorded(uuid::Uuid::new_v4());
    match worker_manager
        .enqueue("experience_scorer", probe_event)
        .await
    {
        Ok(()) => tracing::info!("WorkerManager enqueue verified: targeted job queued"),
        Err(e) => tracing::warn!("WorkerManager enqueue probe failed: {}", e),
    }

    // Verify mark_job_complete against a probe job so the manager-level
    // completion path stays live alongside the queue-level one.
    let probe_job_id = uuid::Uuid::new_v4().to_string();
    match worker_manager.mark_job_complete(&probe_job_id) {
        Ok(()) => tracing::debug!(
            "WorkerManager mark_job_complete verified for probe {}",
            probe_job_id
        ),
        Err(e) => tracing::warn!("mark_job_complete probe failed: {}", e),
    }

    // Remove the isolated probe database directory.
    if let Err(e) = std::fs::remove_dir_all(&probe_dir) {
        tracing::warn!(
            "Worker manager diagnostics cleanup failed for {:?}: {}",
            probe_dir,
            e
        );
    }
    Ok(())
}
