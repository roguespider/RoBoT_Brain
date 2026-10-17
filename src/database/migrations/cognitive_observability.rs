// src/database/migrations/cognitive_observability.rs
// Migration v17: Creates cognitive_observability tables for observability.
// Per Architecture Ch. 27 (Cognitive Monitoring and Observability):
// - cognitive_events table for trace storage
// - decision_records table for decision explanation layer

use anyhow::Result;
use rusqlite::Connection;

/// Run v17 migration: create cognitive observability tables.
pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS cognitive_events (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            correlation_id TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            timestamp INTEGER NOT NULL,
            payload TEXT DEFAULT '{}',
            recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_cognitive_events_correlation
        ON cognitive_events(correlation_id);

        CREATE INDEX IF NOT EXISTS idx_cognitive_events_timestamp
        ON cognitive_events(timestamp);

        CREATE TABLE IF NOT EXISTS decision_records (
            id TEXT PRIMARY KEY,
            correlation_id TEXT NOT NULL,
            decision_type TEXT NOT NULL,
            reasoning TEXT NOT NULL DEFAULT '',
            confidence REAL NOT NULL DEFAULT 0.0,
            timestamp INTEGER NOT NULL,
            payload TEXT DEFAULT '{}'
        );

        CREATE INDEX IF NOT EXISTS idx_decision_records_correlation
        ON decision_records(correlation_id);
        ",
    )?;
    Ok(())
}
