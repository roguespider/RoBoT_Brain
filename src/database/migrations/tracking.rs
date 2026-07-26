// src/database/migrations/tracking.rs
// Migrations 004-005: Events and reputation tracking

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_004_add_events(conn)?;
    migration_005_add_reputations(conn)?;
    Ok(())
}

fn migration_004_add_events(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            description TEXT NOT NULL,
            related_id TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_event_type ON events(event_type);
        ",
    )?;
    Ok(())
}

fn migration_005_add_reputations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS reputations (
            id TEXT PRIMARY KEY,
            score REAL NOT NULL,
            factors TEXT NOT NULL,
            observations INTEGER NOT NULL,
            successes INTEGER NOT NULL,
            failures INTEGER NOT NULL,
            updated_at TEXT NOT NULL,
            history TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_reputation_score ON reputations(score);
        ",
    )?;
    Ok(())
}
