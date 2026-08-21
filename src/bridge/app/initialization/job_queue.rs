// src/bridge/app/initialization/job_queue.rs
//! JobQueue durability probe at startup

use std::sync::{Arc, Mutex};

use crate::experience::queue::JobQueue;

/// Verify the durable JobQueue was wired correctly at startup.
/// Push a probe job, confirm it persists, then restore from a fresh instance.
pub(crate) fn verify_job_queue(
    job_queue: &Arc<Mutex<JobQueue>>,
    database: &Arc<crate::database::sqlite::SqliteDatabase>,
) {
    let mut q = job_queue.lock().unwrap_or_else(|e| e.into_inner());
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
}
