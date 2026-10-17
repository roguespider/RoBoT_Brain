// src/bridge/tools/search/types.rs
// Input types for search tools

use serde::{Deserialize, Serialize};

/// Tool: Full-text search across all data
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GlobalSearchInput {
    pub query: String,
    pub types: Option<Vec<String>>,
    pub limit: Option<usize>,
}

/// Tool: Get recommendations
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetRecommendationsInput {
    pub context: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Get reputation for a target
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetReputationInput {
    pub tool_name: String,
}

/// Input for per-source adapter tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterQueryInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterQuerySubredditInput {
    pub query: String,
    pub subreddit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterQueryTranscriptInput {
    pub query: String,
    pub include_transcript: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterPublicationsInput {
    pub publications: String,
    pub max_posts: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterChannelInput {
    pub channel: String,
    pub max_messages: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSortInput {
    pub query: String,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterLocationInput {
    pub query: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterFilingInput {
    pub query: String,
    pub filing_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterUrlsInput {
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterDoiFormatInput {
    pub doi: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterPlatformsInput {
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterReferencesInput {
    pub references: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterBibliographyInput {
    pub bibliography: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterUrlInput {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterErrorInput {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterQueryInputWithTopic {
    pub query: String,
}

// Web search types
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebSearchInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebOpenInput {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WebExtractInput {
    pub url: String,
}

// Research types
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ResearchInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QuickResearchInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeepResearchInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct FindErrorResolutionInput {
    pub error: String,
}
