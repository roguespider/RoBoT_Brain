// src/database/migrations/cooboploop.rs
// Migration 013: Continuous Objective & Opportunity Loop hardware snapshots.

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS hardware_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_at TEXT DEFAULT (datetime('now')),
            cpu_model TEXT,
            cpu_cores INTEGER,
            memory_total_mb INTEGER,
            memory_available_mb INTEGER,
            storage_total_gb INTEGER,
            storage_available_gb REAL,
            gpu_model TEXT,
            network_interfaces TEXT,
            thermal_state TEXT,
            other TEXT
        );

        CREATE TABLE IF NOT EXISTS strategic_objectives (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT,
            status TEXT DEFAULT 'active',
            priority REAL DEFAULT 0.0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        );",
    )?;
    Ok(())
}
