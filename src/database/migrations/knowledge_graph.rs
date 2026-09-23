// src/database/migrations/knowledge_graph.rs
// Migrations 014-015: Knowledge Graph tables (knowledge_nodes, knowledge_edges)

use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    migration_014_create_knowledge_nodes(conn)?;
    migration_015_create_knowledge_edges(conn)?;
    Ok(())
}

/// Create knowledge_nodes table for storing knowledge items.
fn migration_014_create_knowledge_nodes(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS knowledge_nodes (
            id TEXT PRIMARY KEY,
            label TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'concept',
            confidence REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_node_kind ON knowledge_nodes(kind);
        ",
    )?;
    Ok(())
}

/// Create knowledge_edges table for storing relationships between knowledge nodes.
/// Per Architecture §20.2: knowledge_edges table with index on source_id.
fn migration_015_create_knowledge_edges(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS knowledge_edges (
            id TEXT PRIMARY KEY,
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relationship TEXT NOT NULL,
            confidence REAL NOT NULL DEFAULT 0.5,
            created_at TEXT NOT NULL,
            FOREIGN KEY (source_id) REFERENCES knowledge_nodes(id),
            FOREIGN KEY (target_id) REFERENCES knowledge_nodes(id)
        );
        CREATE INDEX IF NOT EXISTS idx_edge_source ON knowledge_edges(source_id);
        CREATE INDEX IF NOT EXISTS idx_edge_target ON knowledge_edges(target_id);
        ",
    )?;
    Ok(())
}
