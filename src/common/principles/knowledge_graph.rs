//! Knowledge graph persistence — records graph nodes and edges for the
//! knowledge subsystem (Architecture Chapter 20).

use anyhow::Result;
use std::sync::Arc;

use crate::database::queries::knowledge_graph::{
    insert_knowledge_edge, insert_knowledge_graph, insert_knowledge_node, query_edges_by_source,
    query_knowledge_graph_ids, query_nodes_by_kind,
};
use crate::database::sqlite::SqliteDatabase;

/// Insert a knowledge graph entry.
pub async fn insert_graph(
    db: &Arc<SqliteDatabase>,
    id: &str,
    label: &str,
    kind: &str,
) -> Result<()> {
    let conn = db.connection()?;
    insert_knowledge_graph(&conn, id, label, kind)?;
    Ok(())
}

/// Insert a knowledge graph node.
pub async fn insert_node(
    db: &Arc<SqliteDatabase>,
    id: &str,
    label: &str,
    kind: &str,
    confidence: f64,
) -> Result<()> {
    let conn = db.connection()?;
    insert_knowledge_node(&conn, id, label, kind, confidence)?;
    Ok(())
}

/// Insert a knowledge graph edge.
pub async fn insert_edge(
    db: &Arc<SqliteDatabase>,
    id: &str,
    source_id: &str,
    target_id: &str,
    relationship: &str,
    confidence: f64,
) -> Result<()> {
    let conn = db.connection()?;
    insert_knowledge_edge(&conn, id, source_id, target_id, relationship, confidence)?;
    Ok(())
}

/// Query nodes by kind.
pub async fn nodes_by_kind(
    db: &Arc<SqliteDatabase>,
    kind: &str,
) -> Result<Vec<(String, String, String)>> {
    let conn = db.connection()?;
    query_nodes_by_kind(&conn, kind)
}

/// Query edges by source.
pub async fn edges_by_source(
    db: &Arc<SqliteDatabase>,
    source_id: &str,
) -> Result<Vec<(String, String, String, String)>> {
    let conn = db.connection()?;
    query_edges_by_source(&conn, source_id)
}

/// List all knowledge graph IDs.
pub async fn list_graph_ids(db: &Arc<SqliteDatabase>) -> Result<Vec<String>> {
    let conn = db.connection()?;
    query_knowledge_graph_ids(&conn)
}
