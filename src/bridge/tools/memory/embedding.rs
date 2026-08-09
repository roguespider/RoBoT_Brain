//! Vector-index (embedding) tool handlers.
//!
//!

use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

use super::types::{
    DeleteEmbeddingInput, GetEmbeddingInput, ListEmbeddingsInput, SearchSimilarInput,
    StoreEmbeddingInput,
};

/// Execute store embedding tool
pub async fn execute_store_embedding(
    input: StoreEmbeddingInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let memory_uuid = Uuid::parse_str(&input.memory_id)
        .map_err(|e| anyhow::anyhow!("Invalid memory UUID: {}", e))?;

    let model = input.model.unwrap_or_else(|| "default".to_string());

    let embedding =
        crate::database::models::MemoryEmbedding::new(memory_uuid, input.embedding, model);

    let conn = database.connection()?;
    queries::insert_embedding(&conn, &embedding)?;

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "id": embedding.id.to_string(),
        "memory_id": embedding.memory_id.to_string(),
        "dimension": embedding.dimension(),
        "model": embedding.model
    })))
}

/// Execute get embedding tool
pub async fn execute_get_embedding(
    input: GetEmbeddingInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let memory_uuid = Uuid::parse_str(&input.memory_id)
        .map_err(|e| anyhow::anyhow!("Invalid memory UUID: {}", e))?;

    let conn = database.connection()?;

    match queries::get_embedding_by_memory_id(&conn, memory_uuid)? {
        Some(embedding) => Ok(ToolOutput::success(serde_json::json!({
            "found": true,
            "id": embedding.id.to_string(),
            "memory_id": embedding.memory_id.to_string(),
            "dimension": embedding.dimension(),
            "model": embedding.model,
            "embedding": embedding.embedding
        }))),
        None => Ok(ToolOutput::success(serde_json::json!({
            "found": false
        }))),
    }
}

/// Execute search similar tool using cosine similarity
pub async fn execute_search_similar(
    input: SearchSimilarInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(5);
    let min_similarity = input.min_similarity.unwrap_or(0.5);

    let query_embedding = crate::database::models::MemoryEmbedding::new(
        Uuid::new_v4(),
        input.query_embedding,
        "query".to_string(),
    );

    let conn = database.connection()?;
    let embeddings = queries::list_embeddings(&conn, 1000)?;

    let mut similarities: Vec<(String, f32)> = Vec::new();

    for emb in embeddings {
        let similarity = query_embedding.cosine_similarity(&emb);
        if similarity >= min_similarity {
            similarities.push((emb.memory_id.to_string(), similarity));
        }
    }

    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    similarities.truncate(limit);

    Ok(ToolOutput::success(serde_json::json!({
        "results": similarities,
        "count": similarities.len(),
        "query_dimension": query_embedding.dimension()
    })))
}

/// Execute list embeddings tool
pub async fn execute_list_embeddings(
    input: ListEmbeddingsInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(100);

    let conn = database.connection()?;
    let embeddings = queries::list_embeddings(&conn, limit)?;

    let result: Vec<serde_json::Value> = embeddings
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id.to_string(),
                "memory_id": e.memory_id.to_string(),
                "dimension": e.dimension(),
                "model": e.model
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "embeddings": result,
        "count": result.len()
    })))
}

/// Execute delete embedding tool
pub async fn execute_delete_embedding(
    input: DeleteEmbeddingInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let memory_uuid = Uuid::parse_str(&input.memory_id)
        .map_err(|e| anyhow::anyhow!("Invalid memory UUID: {}", e))?;

    let conn = database.connection()?;
    let deleted = queries::delete_embedding_by_memory_id(&conn, memory_uuid)?;

    Ok(ToolOutput::success(serde_json::json!({
        "success": deleted,
        "deleted": deleted
    })))
}

/// Execute get embedding stats tool
pub async fn execute_get_embedding_stats(database: &Arc<SqliteDatabase>) -> Result<ToolOutput> {
    let conn = database.connection()?;
    let count = queries::count_embeddings(&conn)?;

    Ok(ToolOutput::success(serde_json::json!({
        "total_embeddings": count
    })))
}
