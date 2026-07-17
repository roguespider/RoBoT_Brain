// robot/src/experience/hypothesis/support/graph.rs

//! ============================================================================
//! HYPOTHESIS GRAPH
//! ============================================================================
//!
//! Future dependency and relationship graph for hypotheses.
//!
//! This will eventually allow RoBoT to understand connections between beliefs.

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::HypothesisId;

#[derive(Debug, Clone, Default)]
pub struct HypothesisGraph {
    pub nodes: Vec<HypothesisNode>,
    pub edges: Vec<HypothesisEdge>,
}

impl HypothesisGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, hypothesis_id: HypothesisId) {
        self.nodes.push(HypothesisNode { hypothesis_id });
    }

    pub fn add_edge(&mut self, edge: HypothesisEdge) {
        self.edges.push(edge);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisNode {
    pub hypothesis_id: HypothesisId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisEdge {
    pub from: HypothesisId,

    pub to: HypothesisId,

    pub relationship: HypothesisRelationship,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypothesisRelationship {
    Supports,

    Contradicts,

    DependsOn,

    Related,
}
