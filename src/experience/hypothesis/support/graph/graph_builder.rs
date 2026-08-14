// robot/src/experience/hypothesis/support/graph/graph_builder.rs

//! Builder pattern for creating hypothesis graphs.

use super::graph_types::HypothesisRelationship;
use crate::experience::hypothesis::core::hypothesis::HypothesisId;
use crate::experience::hypothesis::support::graph::HypothesisGraph;

/// Builder for creating hypothesis graphs
#[derive(Debug, Clone, Default)]
pub struct GraphBuilder {
    graph: HypothesisGraph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: HypothesisGraph::new(),
        }
    }

    pub fn add_node(mut self, hypothesis_id: HypothesisId) -> Self {
        self.graph.add_node(hypothesis_id);
        self
    }

    pub fn add_support(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::Supports);
        self
    }

    pub fn add_contradiction(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::Contradicts);
        self
    }

    pub fn add_dependency(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::DependsOn);
        self
    }

    pub fn add_related(mut self, from: HypothesisId, to: HypothesisId) -> Self {
        self.graph.add_edge(from, to, HypothesisRelationship::Related);
        self
    }

    /// Add a weighted support edge (confidence-weighted relationship).
    pub fn add_support_weighted(
        mut self,
        from: HypothesisId,
        to: HypothesisId,
        weight: f32,
    ) -> Self {
        self.graph.add_node(from.clone());
        self.graph.add_node(to.clone());
        let edge = super::graph_types::HypothesisEdge::depends_on(from, to)
            .with_weight(weight);
        self.graph.edges.push(edge);
        self
    }

    pub fn build(self) -> HypothesisGraph {
        self.graph
    }
}
