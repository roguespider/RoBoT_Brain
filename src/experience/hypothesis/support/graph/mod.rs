// robot/src/experience/hypothesis/support/graph/mod.rs


//! ============================================================================
//! HYPOTHESIS GRAPH
//! ============================================================================
//!
//! Dependency and relationship graph for hypotheses.
//!
//! This module allows RoBoT to understand connections between beliefs,
//! find relationships, detect cycles, and perform graph analysis.

mod graph_algorithms;
mod graph_builder;
mod graph_types;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::hypothesis::HypothesisId;

// Re-export types for public API
pub use graph_types::{
    EdgeId, GraphStats, HypothesisEdge, HypothesisNode, HypothesisRelationship, NodeMetadata,
};

/// ============================================================================
/// HYPOTHESIS GRAPH
/// ============================================================================
/// A directed graph representing relationships between hypotheses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypothesisGraph {
    #[serde(skip)]
    pub(crate) node_index: HashMap<String, usize>,

    pub(crate) nodes: Vec<HypothesisNode>,

    pub(crate) edges: Vec<HypothesisEdge>,
}

impl HypothesisGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            node_index: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node for a hypothesis
    pub fn add_node(&mut self, hypothesis_id: HypothesisId) -> usize {
        let id_str = hypothesis_id.0.clone();
        if let Some(&idx) = self.node_index.get(&id_str) {
            return idx;
        }

        let node = HypothesisNode {
            hypothesis_id: hypothesis_id.clone(),
            metadata: NodeMetadata::default(),
        };

        let index = self.nodes.len();
        self.nodes.push(node);
        self.node_index.insert(id_str, index);
        index
    }

    /// Add an edge between two hypotheses
    pub fn add_edge(&mut self, from: HypothesisId, to: HypothesisId, relationship: HypothesisRelationship) -> Option<usize> {
        self.add_node(from.clone());
        self.add_node(to.clone());

        if self.has_edge(&from, &to, &relationship) {
            return None;
        }

        let edge = HypothesisEdge {
            id: EdgeId::new(),
            from,
            to,
            relationship,
            weight: 1.0,
        };

        let index = self.edges.len();
        self.edges.push(edge);
        Some(index)
    }

    /// Check if an edge exists
    pub fn has_edge(&self, from: &HypothesisId, to: &HypothesisId, relationship: &HypothesisRelationship) -> bool {
        self.edges.iter().any(|e|
            e.from.0 == from.0 &&
            e.to.0 == to.0 &&
            e.relationship == *relationship
        )
    }

    /// Get all edges for a node
    pub fn get_edges(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter().filter(|e| e.from.0 == hypothesis_id.0).collect()
    }

    /// Get all incoming edges for a node
    pub fn get_incoming_edges(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter().filter(|e| e.to.0 == hypothesis_id.0).collect()
    }

    /// Remove a node and all its edges
    pub fn remove_node(&mut self, hypothesis_id: &HypothesisId) -> bool {
        if let Some(&idx) = self.node_index.get(&hypothesis_id.0) {
            self.nodes.remove(idx);
            self.node_index.remove(&hypothesis_id.0);

            self.node_index.clear();
            for (i, node) in self.nodes.iter().enumerate() {
                self.node_index.insert(node.hypothesis_id.0.clone(), i);
            }

            self.edges.retain(|e| e.from.0 != hypothesis_id.0 && e.to.0 != hypothesis_id.0);

            true
        } else {
            false
        }
    }

    /// Clear all nodes and edges
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.node_index.clear();
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if node exists
    pub fn has_node(&self, hypothesis_id: &HypothesisId) -> bool {
        self.node_index.contains_key(&hypothesis_id.0)
    }

    /// Get node by ID
    pub fn get_node(&self, hypothesis_id: &HypothesisId) -> Option<&HypothesisNode> {
        self.node_index.get(&hypothesis_id.0)
            .and_then(|&idx| self.nodes.get(idx))
    }
}
