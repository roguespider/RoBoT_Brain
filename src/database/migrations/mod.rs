// src/database/migrations/mod.rs
// Database migrations module
// Migrations are split into versioned groups for easier maintenance:
// - v1: Core memory, decisions, and sources (migrations 001-003)
// - v2: Events, reputations, and scheduled tasks (migrations 004-006)
// - v3: Lineage, hypothesis engine, and memory graph (migrations 007-009)
// - v4: Hierarchical memory storage (migrations 010+)

use anyhow::Result;
use rusqlite::Connection;

use crate::database::sqlite::SqliteDatabase;

pub mod core_data_storage;
pub mod tracking;
pub mod scheduling;
pub mod advanced_features;
pub mod hierarchical_memory;
pub mod job_queue;

/// Run all pending migrations.
pub fn run(database: &SqliteDatabase) -> Result<()> {
    let conn = database.connection()?;

    run_migrations(&conn)
}

/// Execute migration sequence.
fn run_migrations(conn: &Connection) -> Result<()> {
    create_migration_table(conn)?;

    let mut version = current_version(conn)?;

    // Run all pending migrations sequentially
    while version < 12 {
        match version {
            0..=2 => {
                core_data_storage::run(conn)?;
                version = 3;
            }
            3..=5 => {
                tracking::run(conn)?;
                scheduling::run(conn)?;
                version = 6;
            }
            6..=8 => {
                advanced_features::run(conn)?;
                version = 9;
            }
            9 | 10 => {
                hierarchical_memory::run(conn)?;
                version = 11;
            }
            11 => {
                job_queue::run(conn)?;
                version = 12;
            }
            _ => break,
        }
        set_version(conn, version)?;
    }

    Ok(())
}

// ==========================================================
// Migration tracking
// ==========================================================

fn create_migration_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_version (

            version INTEGER NOT NULL

        );

        INSERT INTO schema_version(version)

        SELECT 0

        WHERE NOT EXISTS
        (
            SELECT 1 FROM schema_version
        );
        ",
    )?;

    Ok(())
}

fn current_version(conn: &Connection) -> Result<i32> {
    let version = conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0))?;

    Ok(version)
}

fn set_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute("UPDATE schema_version SET version=?1", [version])?;

    Ok(())
}

