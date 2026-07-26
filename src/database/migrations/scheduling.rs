// src/database/migrations/scheduling.rs
// Migration 006: Scheduled tasks persistence

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_006_add_scheduled_tasks(conn)?;
    Ok(())
}

fn migration_006_add_scheduled_tasks(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            task_type TEXT NOT NULL,
            schedule TEXT NOT NULL,
            status TEXT NOT NULL,
            last_run TEXT,
            next_run TEXT,
            failure_count INTEGER DEFAULT 0,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_task_status ON scheduled_tasks(status);
        CREATE INDEX IF NOT EXISTS idx_task_next_run ON scheduled_tasks(next_run);
        ",
    )?;
    Ok(())
}
