//! Migration 20: Worker coordination tables (Architecture §23.8 SQLite Worker Coordination).
//!
//! Creates tables for tracking background worker tasks, status, history, and errors.

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- §23.8: SQLite worker coordination tables
        CREATE TABLE IF NOT EXISTS worker_tasks (
            id TEXT PRIMARY KEY,
            task_type TEXT NOT NULL,
            priority TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            payload TEXT,
            created_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_worker_tasks_status ON worker_tasks(status);
        CREATE INDEX IF NOT EXISTS idx_worker_tasks_type ON worker_tasks(task_type);

        CREATE TABLE IF NOT EXISTS worker_status (
            worker_name TEXT PRIMARY KEY,
            state TEXT NOT NULL,
            last_heartbeat TEXT,
            current_task TEXT
        );

        CREATE TABLE IF NOT EXISTS worker_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            worker TEXT NOT NULL,
            result TEXT NOT NULL,
            duration_ms INTEGER,
            timestamp TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_worker_history_task ON worker_history(task_id);
        CREATE INDEX IF NOT EXISTS idx_worker_history_worker ON worker_history(worker);
        CREATE INDEX IF NOT EXISTS idx_worker_history_timestamp ON worker_history(timestamp);

        CREATE TABLE IF NOT EXISTS worker_errors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            worker_name TEXT NOT NULL,
            task_id TEXT,
            error TEXT NOT NULL,
            timestamp TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_worker_errors_worker ON worker_errors(worker_name);
        CREATE INDEX IF NOT EXISTS idx_worker_errors_timestamp ON worker_errors(timestamp);
        ",
    )?;
    Ok(())
}
