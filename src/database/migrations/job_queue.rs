// src/database/migrations/job_queue.rs
// Migration 012: SQLite-backed job queue (Architecture §23.5 Task Queue)
//
// Replaces the in-memory `JobQueue` (src/experience/queue.rs) so that queued
// jobs survive a process restart. Columns mirror the `Job` struct fields.

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_012_add_job_queue(conn)?;
    Ok(())
}

/// Create the durable job_queue table.
///
/// - `status` mirrors `JobStatus` (pending/running/completed/failed) as TEXT.
/// - `attempts` lets a worker retry before marking a job failed.
/// - `last_error` captures the most recent failure reason for diagnostics.
fn migration_012_add_job_queue(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS job_queue (
            id TEXT PRIMARY KEY,
            experience_id TEXT NOT NULL DEFAULT '',
            observer_name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            last_error TEXT,
            attempts INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Fast lookup of the next pending job for a given observer.
        CREATE INDEX IF NOT EXISTS idx_job_queue_observer_status
            ON job_queue(observer_name, status);

        -- Lookup jobs by parent event.
        CREATE INDEX IF NOT EXISTS idx_job_queue_experience
            ON job_queue(experience_id);

        -- Diagnostic ordering by recency.
        CREATE INDEX IF NOT EXISTS idx_job_queue_updated
            ON job_queue(updated_at);
        ",
    )?;
    Ok(())
}
