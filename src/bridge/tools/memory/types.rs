//! Input type definitions for memory and embedding MCP tools.
//!
//! Each struct corresponds to the JSON schema advertised by
//! [`crate::bridge::tools::memory::definitions`].

use serde::{Deserialize, Serialize};

/// Tool: Store a new memory
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StoreMemoryInput {
    pub content: String,
    pub memory_type: String,
    pub confidence: Option<f32>,
    pub importance: Option<f32>,
    pub tags: Option<Vec<String>>,
}

/// Tool: Search memories
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchMemoryInput {
    pub query: String,
    pub limit: Option<usize>,
}

/// Tool: Get a specific memory
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetMemoryInput {
    pub id: String,
}

/// Tool: List recent memories
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListMemoriesInput {
    pub memory_type: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Archive a memory
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ArchiveMemoryInput {
    pub memory_id: String,
}

/// Tool: Link two memories with a relationship
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LinkMemoriesInput {
    pub from_id: String,
    pub to_id: String,
}

/// Tool: Ranked search across permanent memory
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RankedSearchInput {
    pub query: String,
    pub limit: Option<usize>,
}

/// Tool: Store embedding input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct StoreEmbeddingInput {
    pub memory_id: String,
    pub embedding: Vec<f32>,
    pub model: Option<String>,
}

/// Tool: Get embedding input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetEmbeddingInput {
    pub memory_id: String,
}

/// Tool: Search similar input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SearchSimilarInput {
    pub query_embedding: Vec<f32>,
    pub limit: Option<usize>,
    pub min_similarity: Option<f32>,
}

/// Tool: List embeddings input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct ListEmbeddingsInput {
    pub limit: Option<usize>,
}

/// Tool: Delete embedding input
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteEmbeddingInput {
    pub memory_id: String,
}
