//! Skill input types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegisterSkillInput {
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub examples: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiscoverSkillInput {
    pub name: String,
    pub description: String,
    pub category: String,
    pub source_experience_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSkillInput {
    pub skill_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ListSkillsInput {
    pub category: Option<String>,
    pub enabled_only: Option<bool>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateSkillMasteryInput {
    pub skill_id: String,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GetSkillRecommendationsInput {
    pub limit: Option<u32>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteSkillInput {
    pub skill_id: String,
    pub task: Option<String>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub time_limit_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GetSkillStatsInput {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ApplySkillDecayInput {
    pub decay_rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnableDisableSkillInput {
    pub skill_id: String,
    pub enable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchSkillsInput {
    pub query: String,
    pub category: Option<String>,
    pub min_mastery: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct GetSkillMetricsInput {
    pub skill_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ClearSkillMetricsInput {
    pub skill_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchSkillsByTagInput {
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UnregisterSkillInput {
    pub skill_id: String,
}
