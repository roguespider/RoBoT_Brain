// robot/src/experience/hypothesis/support/graph/graph_types.rs

//! Type definitions for the hypothesis graph.



use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::hypothesis::HypothesisId;

/// ============================================================================
/// GRAPH STATISTICS
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub support_edges: usize,
    pub contradict_edges: usize,
    pub depends_edges: usize,
    pub related_edges: usize,
    pub cycles: usize,
}

/// ============================================================================
/// NODE
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisNode {
    pub hypothesis_id: HypothesisId,
    #[serde(default)]
    pub metadata: NodeMetadata,
}

impl HypothesisNode {
    pub fn new(hypothesis_id: HypothesisId) -> Self {
        Self {
            hypothesis_id,
            metadata: NodeMetadata::default(),
        }
    }
}

/// Node metadata for additional information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub position: Option<(f32, f32)>,
    pub labels: Vec<String>,
    pub weight: f32,
}

impl NodeMetadata {
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some((x, y));
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }
}

/// ============================================================================
/// EDGE
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisEdge {
    pub id: EdgeId,
    pub from: HypothesisId,
    pub to: HypothesisId,
    pub relationship: HypothesisRelationship,
    pub weight: f32,
}

impl HypothesisEdge {
    #[cfg(test)]
    pub fn supports(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Supports,
            weight: 1.0,
        }
    }

    #[cfg(test)]
    pub fn contradicts(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Contradicts,
            weight: 1.0,
        }
    }

    pub fn depends_on(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::DependsOn,
            weight: 1.0,
        }
    }

