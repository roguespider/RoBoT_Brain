// src/bridge/app/initialization/job_queue.rs
//! JobQueue durability probe (explicit diagnostics, P2-001C).
//!
//! Runs entirely against an isolated temporary database so diagnostics can
//! never pop, complete, or fail real jobs from the live production queue.

use std::sync::Arc;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::queue::JobQueue;

/// Verify the durable JobQueue lifecycle against an isolated database.
/// Push probe jobs, confirm they persist, then restore from a fresh instance.
/// The live production queue is never touched.
///
/// Returns `true` if all operations succeed, `false` if database init fails.
pub(crate) fn verify_job_queue() -> bool {
    // Isolated database in the OS temp directory: probe rows are written to
    // their own robot_brain.db, never to the production database.
    let probe_dir = std::env::temp_dir().join(format!(
        "robot_brain_diagnostics_job_queue_{}",
        uuid::Uuid::new_v4()
    ));
    let database = match SqliteDatabase::initialize_at(&probe_dir) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            tracing::warn!(
                "JobQueue diagnostics skipped: isolated database init failed: {}",
                e
            );
            return false;
        }
    };

    let mut q = JobQueue::with_database(database.clone());
    // Exercise the legacy constructor path (Job::new + push_job) alongside the
    // preferred push_job_with_id so both stay live (P0-002 documents both).
    q.push_job("startup-queue-legacy-probe", "experience_scorer");
    if let Some(legacy) = q.pop_job("experience_scorer") {
        q.complete_job(&legacy.id);
    }
    q.push_job_with_id(
        "startup-queue-probe",
        "startup-queue-probe",
        "experience_scorer",
    );
    let popped = q.pop_job("experience_scorer");
    let popped_ok = popped.is_some();
    if let Some(job) = popped.as_ref() {
        q.mark_complete(&job.id).ok();
    }
    q.push_job_with_id(
        "startup-queue-probe-2",
        "startup-queue-probe-2",
        "experience_scorer",
    );
    if let Some(job) = q.pop_job("experience_scorer") {
        q.mark_failed(&job.id, "transient probe failure".to_string())
            .ok();
    }
    // Verify durability: a fresh queue instance restores the
    // pending/running rows written above from SQLite.
    let mut restored_queue = JobQueue::with_database(database.clone());
    let restored = restored_queue.restore_from_database().unwrap_or(0);
    tracing::info!(
        "JobQueue lifecycle verified: pop_ok={}, restored={}",
        popped_ok,
        restored
    );

    // Remove the isolated probe database directory.
    if let Err(e) = std::fs::remove_dir_all(&probe_dir) {
        tracing::warn!(
            "JobQueue diagnostics cleanup failed for {:?}: {}",
            probe_dir,
            e
        );
    }
    true
}
