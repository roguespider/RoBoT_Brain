// src/bridge/tools/search/search.rs
// Core search functions: global_search, get_recommendations, get_reputation.

use std::sync::Arc;

use anyhow::Result;

use crate::bridge::tools::ToolOutput;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;

/// Execute global search tool
pub async fn execute_global_search(
    input: super::types::GlobalSearchInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(20);
    let conn = database.connection()?;

    // Search memories
    let memories = queries::search_memory(&conn, &input.query, limit)?;

    // Categorize results
    let mut memory_results = Vec::new();
    let mut experience_results = Vec::new();

    for m in memories {
        let item = serde_json::json!({
            "id": m.id.to_string(),
            "content": m.content,
            "type": m.memory_type.to_string(),
            "confidence": m.confidence,
            "created_at": m.created_at.to_rfc3339()
        });

        if m.memory_type.to_string() == "experience" {
            experience_results.push(item);
        } else {
            memory_results.push(item);
        }
    }

    let total = memory_results.len() + experience_results.len();

    Ok(ToolOutput::success(serde_json::json!({
        "results": {
            "memories": memory_results,
            "experiences": experience_results,
            "reflections": []
        },
        "total": total,
        "query": input.query
    })))
}

/// Execute get recommendations tool
pub async fn execute_get_recommendations(
    input: super::types::GetRecommendationsInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(5);
    let conn = database.connection()?;

    // Get recent experiences with high confidence
    let experiences = queries::search_memory(&conn, "Experience:", 100)?;

    // Filter high-confidence experiences for recommendations
    let recommendations: Vec<serde_json::Value> = experiences
        .into_iter()
        .filter(|e| e.confidence >= 0.7)
        .take(limit)
        .map(|e| {
            serde_json::json!({
                "type": "experience",
                "id": e.id.to_string(),
                "description": e.content,
                "confidence": e.confidence
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "recommendations": recommendations,
        "based_on": input.context.unwrap_or_else(|| "recent_high_confidence_experiences".to_string())
    })))
}

/// Execute get reputation tool
pub async fn execute_get_reputation(
    input: super::types::GetReputationInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = database.connection()?;

    // Search for mentions of the tool
    let results = queries::search_memory(&conn, &input.tool_name, 100)?;

    // Calculate simple reputation based on mentions
    let total_uses = results.len();
    let high_confidence = results.iter().filter(|r| r.confidence >= 0.7).count();
    let score = if total_uses > 0 {
        high_confidence as f32 / total_uses as f32
    } else {
        0.5
    };

    Ok(ToolOutput::success(serde_json::json!({
        "tool_name": input.tool_name,
        "score": score,
        "success_count": high_confidence,
        "failure_count": total_uses - high_confidence,
        "total_uses": total_uses
    })))
}
