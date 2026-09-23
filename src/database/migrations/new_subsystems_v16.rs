//! Migration 16 — New subsystem tables for v0.0.2.1 architecture.
//!
//! Per Architecture Chapter 22 (Database Design) and gap analysis:
//! Adds tables for execution, tool audit, context lifecycle, retrieval pipeline,
//! confidence system, storage architecture, strategic learning, event router,
//! security audit, developer interface.

use rusqlite::{Connection, Error};

/// Run migration 16.
pub fn run(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        "
        -- Execution engine tables (Chapter 12)
        CREATE TABLE IF NOT EXISTS execution_requests (
            execution_id TEXT PRIMARY KEY,
            plan_id TEXT NOT NULL,
            plan_version TEXT,
            goal_id TEXT,
            actions TEXT,
            dependencies TEXT,
            constraints TEXT,
            permissions TEXT,
            budgets TEXT,
            expected_results TEXT,
            checkpoint_policy TEXT,
            metadata_version TEXT,
            created_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS execution_states (
            execution_id TEXT PRIMARY KEY,
            state TEXT,
            updated_at INTEGER,
            FOREIGN KEY (execution_id) REFERENCES execution_requests(execution_id)
        );

        -- Tool audit table (Chapter 13)
        CREATE TABLE IF NOT EXISTS tool_audits (
            audit_id TEXT PRIMARY KEY,
            tool_id TEXT NOT NULL,
            caller TEXT,
            action TEXT,
            timestamp INTEGER,
            success INTEGER,
            correlation_id TEXT,
            capability_used TEXT
        );

        -- Context lifecycle table (Chapter 15)
        CREATE TABLE IF NOT EXISTS context_lifecycle (
            correlation_id TEXT PRIMARY KEY,
            stage TEXT,
            active INTEGER,
            completed INTEGER,
            disposed INTEGER,
            started_at INTEGER,
            completed_at INTEGER
        );

        -- Retrieval pipeline table (Chapter 16)
        CREATE TABLE IF NOT EXISTS retrieval_queries (
            query_id TEXT PRIMARY KEY,
            original_query TEXT,
            expanded_terms TEXT,
            sources TEXT,
            limit_count INTEGER,
            confidence_threshold REAL,
            created_at INTEGER
        );

        -- Confidence system table (Chapter 19)
        CREATE TABLE IF NOT EXISTS confidence_scores (
            score_id TEXT PRIMARY KEY,
            domain TEXT,
            value REAL,
            source TEXT,
            timestamp INTEGER,
            evidence TEXT,
            correlation_id TEXT
        );

        -- Storage architecture audit (Chapter 21)
        CREATE TABLE IF NOT EXISTS storage_audits (
            audit_id TEXT PRIMARY KEY,
            data_id TEXT,
            layer TEXT,
            action TEXT,
            actor TEXT,
            timestamp INTEGER,
            schema_version TEXT,
            data_version TEXT,
            provenance TEXT
        );

        -- Strategic learning table (Chapter 18)
        CREATE TABLE IF NOT EXISTS strategic_objectives (
            objective_id TEXT PRIMARY KEY,
            name TEXT,
            category TEXT,
            description TEXT,
            priority INTEGER,
            status TEXT,
            created_at INTEGER,
            updated_at INTEGER,
            related_capabilities TEXT
        );

        -- Event router table (Chapter 4)
        CREATE TABLE IF NOT EXISTS routed_events (
            event_id TEXT PRIMARY KEY,
            source_subsystem TEXT,
            target_subsystem TEXT,
            payload TEXT,
            correlation_id TEXT,
            direction TEXT,
            timestamp INTEGER,
            schema_version TEXT,
            provenance TEXT
        );

        -- Security audit table (Chapter 25)
        CREATE TABLE IF NOT EXISTS security_audits (
            audit_id TEXT PRIMARY KEY,
            actor TEXT,
            action TEXT,
            target TEXT,
            timestamp INTEGER,
            success INTEGER,
            confidence_change REAL,
            reason TEXT,
            correlation_id TEXT,
            capability_used TEXT
        );

        -- Developer interface commands (Chapter 28)
        CREATE TABLE IF NOT EXISTS developer_commands (
            command_id TEXT PRIMARY KEY,
            command_name TEXT,
            parameters TEXT,
            timestamp INTEGER,
            result TEXT
        );
        ",
    )?;

    Ok(())
}
