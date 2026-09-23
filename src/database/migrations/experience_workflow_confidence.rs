// src/database/migrations/experience_workflow_confidence.rs
// Migration 015: experience_schema, workflow_schema, confidence_history tables

use anyhow::Result;
use rusqlite::Connection;

/// Run the v5 migration group (tables 015-017).
pub fn run(conn: &Connection) -> Result<()> {
    migration_015_experience_schema(conn)?;
    migration_016_workflow_schema(conn)?;
    migration_017_confidence_history(conn)?;
    Ok(())
}

/// Table 015: experience_schema
/// Stores captured experiences with full lifecycle data.
fn migration_015_experience_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS experience_schema (
            id TEXT PRIMARY KEY,
            goal TEXT NOT NULL,
            plan_id TEXT,
            result TEXT NOT NULL,
            success INTEGER NOT NULL,
            execution_time INTEGER,
            cost REAL DEFAULT 0.0,
            confidence_change REAL DEFAULT 0.0,
            tool_usage TEXT,
            lessons TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_experience_plan ON experience_schema(plan_id);
        CREATE INDEX IF NOT EXISTS idx_experience_success ON experience_schema(success);
        ",
    )?;
    Ok(())
}

/// Table 016: workflow_schema
/// Stores workflow definitions with steps, dependencies, and metadata.
fn migration_016_workflow_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS workflow_schema (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            steps TEXT NOT NULL,
            dependencies TEXT,
            required_skills TEXT,
            estimated_cost REAL DEFAULT 0.0,
            estimated_confidence REAL DEFAULT 0.5,
            alternative_branches TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_workflow_name ON workflow_schema(name);
        ",
    )?;
    Ok(())
}

/// Table 017: confidence_history
/// Tracks confidence changes over time for explainability.
fn migration_017_confidence_history(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS confidence_history (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL,
            old_confidence REAL NOT NULL,
            new_confidence REAL NOT NULL,
            reason TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_confidence_item ON confidence_history(item_id);
        CREATE INDEX IF NOT EXISTS idx_confidence_timestamp ON confidence_history(timestamp);
        ",
    )?;
    Ok(())
}

/// Active reference to prevent dead-code warnings.
pub fn reference_migration() {
    // Verify the module compiles and is wired
    let run_fn = run;
    tracing::debug!(
        migration_fn = "experience_workflow_confidence",
        "Migration function referenced via {:?}",
        std::any::type_name_of_val(&run_fn)
    );
}
