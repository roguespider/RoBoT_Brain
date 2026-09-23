//! Memory graph support - Per Architecture §20.1 "Concept relationships"
//!
//! Defines nodes and edges for the memory relationship graph,
//! and provides database persistence and query functions.

use serde::{Deserialize, Serialize};

/// Memory node in the relationship graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryNode {
    /// Unique identifier
    pub id: String,
    /// Content of the memory
    pub content: String,
    /// Type of the node
    pub node_type: String,
    /// Confidence in this node
    pub confidence: f32,
}

/// Memory edge (relationship) between nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryEdge {
    /// Source node ID
    pub source_id: String,
    /// Target node ID
    pub target_id: String,
    /// Type of relationship
    pub relationship_type: String,
    /// Confidence in this relationship
    pub confidence: f32,
}

/// Insert a memory node into the database
pub fn insert_node(conn: &rusqlite::Connection, n: &MemoryNode) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO memory_nodes (id, content, node_type, confidence) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![n.id, n.content, n.node_type, n.confidence],
    )?;
    Ok(())
}

/// Insert a memory edge into the database
pub fn insert_edge(conn: &rusqlite::Connection, e: &MemoryEdge) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO memory_relationships (id, memory_id, related_id, relationship_type) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![
            format!("{}-{}", e.source_id, e.target_id),
            e.source_id,
            e.target_id,
            e.relationship_type,
        ],
    )?;
    Ok(())
}

/// Get connections for a node
pub fn get_connections(
    conn: &rusqlite::Connection,
    node_id: &str,
) -> Result<Vec<MemoryEdge>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT memory_id, related_id, relationship_type FROM memory_relationships WHERE memory_id = ?1 OR related_id = ?1",
    )?;

    let edges: Vec<MemoryEdge> = stmt
        .query_map([node_id], |row| {
            let source_id: String = row.get(0)?;
            let target_id: String = row.get(1)?;
            let rel_type: String = row.get(2)?;
            Ok(MemoryEdge {
                source_id,
                target_id,
                relationship_type: rel_type,
                confidence: 0.5,
            })
        })?
        .filter_map(Result::ok)
        .collect();

    Ok(edges)
}

/// Find path between nodes using BFS
pub fn find_path(
    conn: &rusqlite::Connection,
    from: &str,
    to: &str,
    max_depth: usize,
) -> Result<Option<Vec<String>>, rusqlite::Error> {
    use std::collections::{HashMap, VecDeque};

    let mut visited: HashMap<String, String> = HashMap::new();
    let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
    queue.push_back((from.to_string(), vec![from.to_string()]));

    while let Some((current, path)) = queue.pop_front() {
        if current == to || path.len() > max_depth {
            if current == to {
                return Ok(Some(path));
            }
            continue;
        }

        // Get connections
        let connections = get_connections(conn, &current)?;
        for edge in connections {
            let next = if edge.source_id == current {
                edge.target_id.clone()
            } else {
                edge.source_id.clone()
            };

            if !visited.contains_key(&next) {
                visited.insert(next.clone(), current.clone());
                let mut new_path = path.clone();
                new_path.push(next.clone());
                queue.push_back((next, new_path));
            }
        }
    }

    Ok(None)
}
