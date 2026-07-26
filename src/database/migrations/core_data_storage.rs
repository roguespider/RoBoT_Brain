// src/database/migrations/core_data_storage.rs
// Migrations 001-003: Core memory and decision tables

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_001_initial_memory(conn)?;
    migration_002_add_decision_memory(conn)?;
    migration_003_add_memory_sources(conn)?;
    Ok(())
}

fn migration_001_initial_memory(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            memory_type TEXT NOT NULL,
            confidence REAL DEFAULT 0.5,
            importance REAL DEFAULT 0.5,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_memory_type ON memories(memory_type);
        ",
    )?;
    Ok(())
}

fn migration_002_add_decision_memory(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS decisions (
            id TEXT PRIMARY KEY,
            task TEXT NOT NULL,
            chosen_workflow TEXT NOT NULL,
            alternatives TEXT,
            reasoning TEXT,
            result TEXT,
            success INTEGER,
            confidence REAL DEFAULT 0.5,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_decision_task ON decisions(task);
        ",
    )?;
    Ok(())
}

fn migration_003_add_memory_sources(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS memory_sources (
            id TEXT PRIMARY KEY,
            memory_id TEXT NOT NULL,
            source_type TEXT,
            source_name TEXT,
            source_location TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_source_memory ON memory_sources(memory_id);
        ",
    )?;
    Ok(())
}
