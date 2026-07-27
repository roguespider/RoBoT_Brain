// robot/src/experience/hypothesis/support/graph/graph_algorithms.rs


//! Graph algorithm implementations for hypothesis graphs.

use std::collections::{HashMap, HashSet, VecDeque};

use super::graph_types::{GraphStats, HypothesisEdge, HypothesisRelationship};
use crate::experience::hypothesis::core::hypothesis::HypothesisId;
use crate::experience::hypothesis::support::graph::HypothesisGraph;

impl HypothesisGraph {
    /// Find all connected hypotheses
    pub fn find_connected(&self, hypothesis_id: &HypothesisId) -> Vec<HypothesisId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(hypothesis_id.0.clone());
        visited.insert(hypothesis_id.0.clone());

        while let Some(current) = queue.pop_front() {
            for edge in self.edges.iter().filter(|e| e.from.0 == current) {
                if visited.insert(edge.to.0.clone()) {
                    result.push(HypothesisId(edge.to.0.clone()));
                    queue.push_back(edge.to.0.clone());
                }
            }

            for edge in self.edges.iter().filter(|e| e.to.0 == current) {
                if visited.insert(edge.from.0.clone()) {
                    result.push(HypothesisId(edge.from.0.clone()));
                    queue.push_back(edge.from.0.clone());
                }
            }
        }

        result
    }

    /// Find path between two hypotheses using BFS
    pub fn find_path(&self, from: &HypothesisId, to: &HypothesisId) -> Option<Vec<HypothesisId>> {
        if !self.node_index.contains_key(&from.0) || !self.node_index.contains_key(&to.0) {
            return None;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<String, Option<String>> = HashMap::new();

        queue.push_back(from.0.clone());
        visited.insert(from.0.clone());
        parent.insert(from.0.clone(), None);

        while let Some(current) = queue.pop_front() {
            if current == to.0 {
                let mut path = Vec::new();
                let mut node = Some(current);
                while let Some(n) = node {
                    path.push(HypothesisId(n.clone()));
                    node = parent.get(&n).cloned().flatten();
                }
                path.reverse();
                return Some(path);
            }

            for edge in self.edges.iter().filter(|e| e.from.0 == current) {
                if visited.insert(edge.to.0.clone()) {
                    parent.insert(edge.to.0.clone(), Some(current.clone()));
                    queue.push_back(edge.to.0.clone());
                }
            }
        }

        None
    }

    /// Detect cycles in the graph
    pub fn detect_cycles(&self) -> Vec<Vec<HypothesisId>> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut cycles = Vec::new();
        let mut path = Vec::new();

        for node in &self.nodes {
            if !visited.contains(&node.hypothesis_id.0) {
                self.detect_cycles_dfs(
                    &node.hypothesis_id,
                    &mut visited,
                    &mut recursion_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn detect_cycles_dfs(
        &self,
        hypothesis_id: &HypothesisId,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<HypothesisId>>,
    ) {
        let id = hypothesis_id.0.clone();
        visited.insert(id.clone());
        recursion_stack.insert(id.clone());
        path.push(id.clone());

        for edge in self.edges.iter().filter(|e| e.from.0 == id) {
            if !visited.contains(&edge.to.0) {
                self.detect_cycles_dfs(
                    &HypothesisId(edge.to.0.clone()),
                    visited,
                    recursion_stack,
                    path,
                    cycles,
                );
            } else if recursion_stack.contains(&edge.to.0) {
                if let Some(start) = path.iter().position(|p| p == &edge.to.0) {
                    let cycle: Vec<HypothesisId> = path[start..]
                        .iter()
                        .chain(std::iter::once(&edge.to.0))
                        .map(|s| HypothesisId(s.clone()))
                        .collect();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        recursion_stack.remove(&id);
    }

    /// Find all supporting edges for a hypothesis
    pub fn find_supporters(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter()
            .filter(|e| e.to.0 == hypothesis_id.0 && e.relationship == HypothesisRelationship::Supports)
            .collect()
    }

    /// Find all contradicting edges for a hypothesis
    pub fn find_contradictions(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter()
            .filter(|e| e.to.0 == hypothesis_id.0 && e.relationship == HypothesisRelationship::Contradicts)
            .collect()
    }

    /// Find all dependencies for a hypothesis
    pub fn find_dependencies(&self, hypothesis_id: &HypothesisId) -> Vec<&HypothesisEdge> {
        self.edges.iter()
            .filter(|e| e.from.0 == hypothesis_id.0 && e.relationship == HypothesisRelationship::DependsOn)
            .collect()
    }

    /// Get strongly connected components
    pub fn strongly_connected_components(&self) -> Vec<Vec<HypothesisId>> {
        let mut visited = HashSet::new();
        let mut finish_order = Vec::new();

        for node in &self.nodes {
            if !visited.contains(&node.hypothesis_id.0) {
                self.dfs_fill_order(&node.hypothesis_id, &mut visited, &mut finish_order);
            }
        }

        let transposed = self.transpose();

        visited.clear();
        let mut components = Vec::new();

        for id in finish_order.into_iter().rev() {
            if !visited.contains(&id) {
                let mut component = Vec::new();
                transposed.dfs_collect(&HypothesisId(id.clone()), &mut visited, &mut component);
                components.push(component);
            }
        }

        components
    }

    fn dfs_fill_order(&self, hypothesis_id: &HypothesisId, visited: &mut HashSet<String>, finish_order: &mut Vec<String>) {
        visited.insert(hypothesis_id.0.clone());

        for edge in self.edges.iter().filter(|e| e.from.0 == hypothesis_id.0) {
            if !visited.contains(&edge.to.0) {
                self.dfs_fill_order(&HypothesisId(edge.to.0.clone()), visited, finish_order);
            }
        }

        finish_order.push(hypothesis_id.0.clone());
    }

    fn dfs_collect(&self, hypothesis_id: &HypothesisId, visited: &mut HashSet<String>, result: &mut Vec<HypothesisId>) {
        visited.insert(hypothesis_id.0.clone());
        result.push(hypothesis_id.clone());

        for edge in self.edges.iter().filter(|e| e.from.0 == hypothesis_id.0) {
            if !visited.contains(&edge.to.0) {
                self.dfs_collect(&HypothesisId(edge.to.0.clone()), visited, result);
            }
        }
    }

    fn transpose(&self) -> Self {
        let mut transposed = Self::new();

        for node in &self.nodes {
            transposed.add_node(node.hypothesis_id.clone());
        }

        for edge in &self.edges {
            transposed.add_edge(
                edge.to.clone(),
                edge.from.clone(),
                edge.relationship,
            );
        }

        transposed
    }

    /// Get the topological order of hypotheses
    pub fn topological_sort(&self) -> Option<Vec<HypothesisId>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        for node in &self.nodes {
            in_degree.insert(node.hypothesis_id.0.clone(), 0);
            adjacency.insert(node.hypothesis_id.0.clone(), Vec::new());
        }

        for edge in &self.edges {
            adjacency.entry(edge.from.0.clone())
                .or_default()
                .push(edge.to.0.clone());
            *in_degree.entry(edge.to.0.clone()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|item| *item.1 == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(HypothesisId(node.clone()));

            if let Some(neighbors) = adjacency.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None
        }
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        let support_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::Supports)
            .count();
        let contradict_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::Contradicts)
            .count();
        let depends_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::DependsOn)
            .count();
        let related_edges = self.edges.iter()
            .filter(|e| e.relationship == HypothesisRelationship::Related)
            .count();

        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            support_edges,
            contradict_edges,
            depends_edges,
            related_edges,
            cycles: self.detect_cycles().len(),
        }
    }
}
