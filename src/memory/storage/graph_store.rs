//! Graph store — per Architecture §14 (Memory Hierarchy).
//! Provides graph-based relationship storage for memory records.

use serde_json;
use std::collections::HashMap;

/// Graph store for relationship-based memory storage.
#[derive(Default)]
pub struct GraphStore {
    nodes: HashMap<String, serde_json::Value>,
    edges: Vec<(String, String, String)>,
}

impl GraphStore {
    /// Create a new graph store.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Store a node with its associated data.
    pub fn add_node(&mut self, key: String, value: serde_json::Value) {
        self.nodes.insert(key, value);
    }

    /// Add a directed edge from source to target with a relationship label.
    pub fn add_edge(&mut self, source: String, target: String, relationship: String) {
        self.edges.push((source, target, relationship));
    }

    /// Get a node by key.
    pub fn get_node(&self, key: &str) -> Option<&serde_json::Value> {
        self.nodes.get(key)
    }

    /// Get all edges from a source node.
    pub fn get_edges(&self, source: &str) -> Vec<&(String, String, String)> {
        self.edges.iter().filter(|(s, _, _)| s == source).collect()
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
