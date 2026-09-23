//! Execution Graph — Action dependency graph (Architecture Chapter 12.7).
//!
//! Wiring: `execution/` -> `workflows/` -> `database/`

/// A node in the execution graph representing an executable action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionGraphNode {
    /// Node identifier.
    pub id: String,
    /// Action name.
    pub action: String,
    /// Dependencies (other node IDs).
    pub dependencies: Vec<String>,
    /// Whether this node is completed.
    pub completed: bool,
}

impl ActionGraphNode {
    /// Create a new graph node.
    pub fn new(id: &str, action: &str) -> Self {
        Self {
            id: id.to_string(),
            action: action.to_string(),
            dependencies: Vec::new(),
            completed: false,
        }
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, dependency: &str) {
        if !self.dependencies.contains(&dependency.to_string()) {
            self.dependencies.push(dependency.to_string());
        }
    }

    /// Mark as completed.
    pub fn complete(&mut self) {
        self.completed = true;
    }
}

/// The execution graph connects actions through dependencies.
#[derive(Debug, Clone, Default)]
pub struct ExecutionGraph {
    /// Nodes in the graph.
    pub nodes: std::collections::HashMap<String, ActionGraphNode>,
}

impl ExecutionGraph {
    /// Create a new execution graph.
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, node: ActionGraphNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &str) -> Option<&ActionGraphNode> {
        self.nodes.get(id)
    }

    /// Get mutable node by ID.
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut ActionGraphNode> {
        self.nodes.get_mut(id)
    }

    /// Check if all nodes are completed.
    pub fn is_complete(&self) -> bool {
        !self.nodes.is_empty() && self.nodes.values().all(|n| n.completed)
    }

    /// Get nodes that are ready to execute (all dependencies completed).
    pub fn ready_nodes(&self) -> Vec<String> {
        let mut ready = Vec::new();
        for (id, node) in &self.nodes {
            if node.completed {
                continue;
            }
            let all_deps_complete = node
                .dependencies
                .iter()
                .all(|dep_id| self.nodes.get(dep_id).map(|n| n.completed).unwrap_or(false));
            if all_deps_complete {
                ready.push(id.clone());
            }
        }
        ready
    }
}
