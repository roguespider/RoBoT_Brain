// src/database/queries/knowledge_graph.rs
//! Query functions for knowledge_graph, knowledge_nodes, and knowledge_edges tables.

use anyhow::Result;
use rusqlite::{Connection, params};

/// Insert a knowledge_graph record.
pub fn insert_knowledge_graph(conn: &Connection, id: &str, label: &str, kind: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO knowledge_graph (id, label, kind) VALUES (?1, ?2, ?3)",
        [id, label, kind],
    )?;
    Ok(())
}

/// Insert a knowledge_node record.
pub fn insert_knowledge_node(
    conn: &Connection,
    id: &str,
    label: &str,
    kind: &str,
    confidence: f64,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO knowledge_nodes (id, label, kind, confidence) VALUES (?1, ?2, ?3, ?4)",
        params![id, label, kind, confidence],
    )?;
    Ok(())
}

/// Insert a knowledge_edge record.
pub fn insert_knowledge_edge(
    conn: &Connection,
    id: &str,
    source_id: &str,
    target_id: &str,
    relationship: &str,
    confidence: f64,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO knowledge_edges (id, source_id, target_id, relationship, confidence) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, source_id, target_id, relationship, confidence],
    )?;
    Ok(())
}

/// Query knowledge nodes by kind.
pub fn query_nodes_by_kind(conn: &Connection, kind: &str) -> Result<Vec<(String, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, label, kind FROM knowledge_nodes WHERE kind = ?1")?;
    let rows = stmt.query_map([kind], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query knowledge edges by source node.
pub fn query_edges_by_source(
    conn: &Connection,
    source_id: &str,
) -> Result<Vec<(String, String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_id, target_id, relationship FROM knowledge_edges WHERE source_id = ?1",
    )?;
    let rows = stmt.query_map([source_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Query all knowledge graph IDs.
pub fn query_knowledge_graph_ids(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM knowledge_graph")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
