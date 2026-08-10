//! Search and ranked-search tool handlers.

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::tools::ToolOutput;
use crate::database::models::{MemoryCard, Observation};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{
    Experience, ExperienceContext, ExperienceOutcome, ExperienceType,
};
use crate::memory::MemoryRetrieval;

use super::super::types::{RankedSearchInput, SearchMemoryInput};

/// Execute search memory tool
/// Per Architecture §07: Memory access generates observations for the learning pipeline.
/// Per Architecture §6.3: Uses MemoryRetrieval service
pub async fn execute_search_memory(
    input: SearchMemoryInput,
    database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(10);

    let results = memory_retrieval.retrieve(&input.query).await;
    let results: Vec<_> = results.into_iter().take(limit).collect();

    let query_preview = if input.query.len() > 50 {
        format!("{}...", &input.query[..50])
    } else {
        input.query.clone()
    };
    let observation = Observation::new(
        format!("Searched for: {}", query_preview),
        format!("results_found={}", results.len()),
        "memory_lookup".to_string(),
    );
    let conn = database.connection()?;
    queries::insert_observation(&conn, &observation)?;

    let mut experience = Experience::new(
        format!("Memory lookup: {}", query_preview),
        format!(
            "Searched memory with query '{}', found {} results",
            input.query,
            results.len()
        ),
        ExperienceType::MemoryLookup,
        vec![observation.id],
    );
    experience.context = ExperienceContext {
        search_query: Some(input.query.clone()),
        results_count: Some(results.len()),
        source: Some("search_memory_tool".to_string()),
        ..Default::default()
    };
    experience.outcome = ExperienceOutcome::success();
    experience.tags = vec!["memory".to_string(), "search".to_string()];
    if let Err(e) = experience.commit() {
        tracing::warn!("Experience already committed: {}", e);
    }
    let memory_from_exp = MemoryCard::from_experience(&experience);
    queries::insert_memory(&conn, &memory_from_exp)?;

    let memories: Vec<serde_json::Value> = results
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.item.id.to_string(),
                "content": r.item.content,
                "memory_type": r.item.memory_type.to_string(),
                "layer": r.item.layer.to_string(),
                "relevance_score": r.relevance_score,
                "confidence": r.item.confidence,
                "importance": r.item.importance,
                "created_at": r.item.created_at.to_rfc3339(),
                "accessed_at": r.item.accessed_at.to_rfc3339()
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "results": memories,
        "count": memories.len(),
        "observation_id": observation.id.to_string(),
        "experience_id": experience.id.to_string()
    })))
}

/// Execute ranked search tool
pub async fn execute_ranked_search(
    input: RankedSearchInput,
    results: Vec<(crate::memory::types::MemoryItem, f32)>,
) -> Result<ToolOutput> {
    let serialized: Vec<serde_json::Value> = results
        .iter()
        .map(|(item, score)| serde_json::json!({
            "memory": item,
            "relevance_score": score,
        }))
        .collect();
    Ok(ToolOutput::success(serde_json::json!({
        "query": input.query,
        "results": serialized,
        "count": results.len(),
    })))
}
