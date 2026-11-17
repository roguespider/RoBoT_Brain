// src/database/migrations/security.rs
// Migration 019: Security tables (Architecture §25)

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_019_security_tables(conn)?;
    Ok(())
}

fn migration_019_security_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS security_audits (
            audit_id TEXT PRIMARY KEY,
            actor TEXT NOT NULL,
            action TEXT NOT NULL,
            target TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            success INTEGER NOT NULL DEFAULT 1,
            confidence_change REAL DEFAULT 0.0,
            reason TEXT,
            correlation_id TEXT,
            capability_used TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_security_audits_actor ON security_audits(actor);
        CREATE INDEX IF NOT EXISTS idx_security_audits_action ON security_audits(action);
        CREATE INDEX IF NOT EXISTS idx_security_audits_timestamp ON security_audits(timestamp);

        CREATE TABLE IF NOT EXISTS capability_grants (
            id TEXT PRIMARY KEY,
            actor_id TEXT NOT NULL,
            capability TEXT NOT NULL,
            scope TEXT NOT NULL DEFAULT 'global',
            granted_at INTEGER NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            UNIQUE(actor_id, capability, scope)
        );
        CREATE INDEX IF NOT EXISTS idx_capability_grants_actor ON capability_grants(actor_id);
        CREATE INDEX IF NOT EXISTS idx_capability_grants_capability ON capability_grants(capability);
        ",
    )?;
    Ok(())
}
