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
    pub fn supports(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Supports,
            weight: 1.0,
        }
    }

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

    pub fn related(from: HypothesisId, to: HypothesisId) -> Self {
        Self {
            id: EdgeId::new(),
            from,
            to,
            relationship: HypothesisRelationship::Related,
            weight: 1.0,
        }
    }
}

/// Unique edge identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

impl EdgeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// RELATIONSHIP
/// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HypothesisRelationship {
    /// Hypothesis A provides evidence supporting Hypothesis B
    Supports,

    /// Hypothesis A provides evidence contradicting Hypothesis B
    Contradicts,

    /// Hypothesis A depends on Hypothesis B being true
    DependsOn,

    /// Hypothesis A is related to Hypothesis B
    Related,
}

impl HypothesisRelationship {
    pub fn is_supporting(&self) -> bool {
        matches!(self, HypothesisRelationship::Supports)
    }

    pub fn is_contradicting(&self) -> bool {
        matches!(self, HypothesisRelationship::Contradicts)
    }

    pub fn inverse(&self) -> Self {
        match self {
            HypothesisRelationship::Supports => HypothesisRelationship::Contradicts,
            HypothesisRelationship::Contradicts => HypothesisRelationship::Supports,
            HypothesisRelationship::DependsOn => HypothesisRelationship::DependsOn,
            HypothesisRelationship::Related => HypothesisRelationship::Related,
        }
    }
}
