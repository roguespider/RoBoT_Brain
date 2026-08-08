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
    pub title: String,
    pub description: String,
    pub reflection_type: String,
    pub experience_ids: Vec<String>,
}

/// Tool: Analyze patterns
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AnalyzePatternsInput {
    pub experience_ids: Vec<String>,
}

/// Tool: Get pattern summary
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetPatternsInput {
    pub min_confidence: Option<f32>,
    pub pattern_type: Option<String>,
}
