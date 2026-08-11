// src/tools/reflection/types.rs
//! Reflection tool input types

use serde::{Deserialize, Serialize};

/// Tool: Get insights
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetInsightsInput {
    pub min_confidence: Option<f32>,
    pub limit: Option<usize>,
}

/// Tool: Create a reflection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateReflectionInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub reflection_type: Option<String>,
    pub experience_ids: Option<Vec<String>>,
}

/// Tool: Analyze patterns
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AnalyzePatternsInput {
    pub experience_ids: Option<Vec<String>>,
}

/// Tool: Get pattern summary
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetPatternsInput {
    pub min_confidence: Option<f32>,
    pub pattern_type: Option<String>,
}

/// Tool: Validate a reflection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ValidateReflectionInput {
    pub reflection_id: String,
}

/// Tool: List reflections by status
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListReflectionsByStatusInput {
    pub status: String,
}

/// Tool: Update a reflection
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateReflectionInput {
    pub reflection_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
}
