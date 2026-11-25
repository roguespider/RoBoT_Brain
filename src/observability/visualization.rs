//! Observability Visualization — Per Architecture Chapter 27.12 (Cognitive Visualization).
//!
//! Provides data structures for visualizing internal RoBoT activity,
//! including memory retrieval paths, decision history, and worker activity.

/// A visualization node representing a subsystem or component.
#[derive(Debug, Clone)]
pub struct VizNode {
    /// Unique node identifier.
    pub id: String,
    /// Human-readable node label.
    pub label: String,
    /// Node type (e.g., "memory", "planning", "execution").
    pub node_type: String,
}

/// A visualization edge representing a relationship between nodes.
#[derive(Debug, Clone)]
pub struct VizEdge {
    /// Unique edge identifier.
    pub id: String,
    /// Source node identifier.
    pub source: String,
    /// Target node identifier.
    pub target: String,
    /// Edge label describing the relationship.
    pub label: String,
}

/// A visualization graph representing cognitive activity.
#[derive(Debug, Clone, Default)]
pub struct VizGraph {
    /// All nodes in the graph.
    pub nodes: Vec<VizNode>,
    /// All edges in the graph.
    pub edges: Vec<VizEdge>,
}

impl VizGraph {
    /// Create a new empty visualization graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: VizNode) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: VizEdge) {
        self.edges.push(edge);
    }

    /// Count total nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Count total edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}
