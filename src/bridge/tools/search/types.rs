// src/bridge/tools/search/types.rs
// Input structs for search-related tools.

use serde::{Deserialize, Serialize};

// Core search tools
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GlobalSearchInput {
    pub query: String,
    pub types: Option<Vec<String>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetRecommendationsInput {
    pub context: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetReputationInput {
    pub tool_name: String,
}

// Web interaction tools
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

// Research pipeline tools
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

// Per-source adapter input types (research_engine.md Tool Catalog, R11)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterQueryInput {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterUrlInput {
    pub url: String,
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
pub struct AdapterPlatformsInput {
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterUrlsInput {
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSubredditInput {
    pub query: String,
    pub subreddit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterYouTubeInput {
    pub query: String,
    pub include_transcript: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSubstackInput {
    pub publications: String,
    pub max_posts: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterBlueskyInput {
    pub query: String,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterTelegramInput {
    pub channel: String,
    pub max_messages: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterOsmInput {
    pub query: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterSecFilingsInput {
    pub query: String,
    pub filing_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AdapterCitationsInput {
    pub doi: String,
    pub format: String,
}
