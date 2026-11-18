// src/api/bridge/tools/knowledge/types.rs
//! Knowledge tool input types

use serde::{Deserialize, Serialize};

/// Tool: Add new knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddKnowledgeInput {
    /// The knowledge statement
    pub statement: String,
    /// Type of knowledge
    pub knowledge_type: Option<String>,
    /// Initial confidence (0.0 - 1.0)
    pub confidence: Option<f32>,
    /// Tags for categorization
    pub tags: Option<Vec<String>>,
    /// Source of this knowledge
    pub source: Option<String>,
}

/// Tool: Query knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QueryKnowledgeInput {
    /// Text search
    pub query: String,
    /// Filter by type
    pub knowledge_type: Option<String>,
    /// Minimum confidence
    pub min_confidence: Option<f32>,
    /// Only mature knowledge
    pub mature_only: Option<bool>,
    /// Maximum results
    pub limit: Option<usize>,
}

/// Tool: Record knowledge application
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordKnowledgeApplicationInput {
    /// Knowledge ID that was applied
    pub knowledge_id: String,
    /// Whether application was successful
    pub success: bool,
}

/// Tool: Get knowledge statistics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetKnowledgeStatsInput {
    // No parameters needed
}

/// Tool: Get mature knowledge
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetMatureKnowledgeInput {
    pub limit: Option<usize>,
}

/// Tool: Update an existing knowledge item
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateKnowledgeInput {
    /// ID of the knowledge item to update
    pub knowledge_id: String,
    /// Updated statement
    pub statement: Option<String>,
    /// Updated confidence (0.0 - 1.0)
    pub confidence: Option<f32>,
    /// Updated tags (replaces existing)
    pub tags: Option<Vec<String>>,
}

/// Tool: Delete a knowledge item
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteKnowledgeInput {
    /// ID of the knowledge item to delete
    pub knowledge_id: String,
}

/// Tool: Get knowledge related to a given item
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetRelatedKnowledgeInput {
    /// ID of the knowledge item to find relations for
    pub knowledge_id: String,
}

/// Tool: Validate all knowledge dependencies
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct ValidateKnowledgeDependenciesInput {}

/// Tool: Bump the version of a knowledge item
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BumpKnowledgeVersionInput {
    /// ID of the knowledge item to bump
    pub knowledge_id: String,
    /// Version bump type: major, minor, or patch
    pub bump_type: Option<String>,
    /// Initial version string (used if version tracking not yet initialized)
    pub initial_version: Option<String>,
}

/// Tool: Set knowledge status (activate, suspend, disprove)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SetKnowledgeStatusInput {
    /// ID of the knowledge item
    pub knowledge_id: String,
    /// Action: activate, suspend, or disprove
    pub action: String,
}

/// Tool: Manage knowledge dependencies
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManageKnowledgeDependencyInput {
    /// ID of the knowledge item
    pub knowledge_id: String,
    /// Action: add, remove, get, or impact
    pub action: String,
    /// ID of the dependency target (for add/remove)
    pub depends_on_id: Option<String>,
    /// Dependency type for add: required, optional, conflict, or replaces
    pub dependency_type: Option<String>,
    /// Optional version constraint for add (e.g. ">=1.0.0")
    pub version_constraint: Option<String>,
}

/// Tool: Add a relation between two knowledge items
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddKnowledgeRelationInput {
    /// ID of the source knowledge item
    pub knowledge_id: String,
    /// ID of the target knowledge item
    pub related_id: String,
    /// Relation type: related, supports, contradicts, specializes, generalizes, prerequisite
    pub relation_type: Option<String>,
    /// Confidence in the relation (0.0-1.0)
    pub confidence: Option<f32>,
}

/// Tool: Search knowledge by tag or get items needing review
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct SearchKnowledgeByTagInput {
    /// Tag to search for (if omitted, returns items needing review)
    pub tag: Option<String>,
}
