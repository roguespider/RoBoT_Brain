//! Get, list, archive, and link tool handlers.

use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::database::models::{MemoryCard, Observation};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType};
use crate::memory::MemoryRetrieval;

use super::super::types::{
    ArchiveMemoryInput, DeleteMemoryInput, GetMemoryInput, LinkMemoriesInput, ListMemoriesInput,
};

/// Execute get memory tool
/// Per Architecture §6.3: Uses MemoryRetrieval service
pub async fn execute_get_memory(
    input: GetMemoryInput,
    database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let uuid = Uuid::parse_str(&input.id).map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;

    let working = memory_retrieval.working_memory().retrieve(&uuid).await;
    let permanent = memory_retrieval.permanent_memory().retrieve(&uuid).await;

    match working.or(permanent) {
        Some(m) => {
            let conn = database.connection()?;

            let content_preview = if m.content.len() > 50 {
                format!("{}...", &m.content[..50])
            } else {
                m.content.clone()
            };
            let observation = Observation::new(
                format!("Retrieved memory: {}", content_preview),
                format!(
                    "memory_type={}, id={}, layer={}",
                    m.memory_type, m.id, m.layer
                ),
                "memory_retrieval".to_string(),
            );
            queries::insert_observation(&conn, &observation)?;

            let mut experience = Experience::new(
                format!("Memory retrieved: {}", content_preview),
                format!(
                    "Retrieved {} memory with id {} from {}",
                    m.memory_type, m.id, m.layer
                ),
                ExperienceType::MemoryLookup,
                vec![observation.id],
            );
            experience.context = ExperienceContext {
                memory_type: Some(m.memory_type.to_string()),
                content_length: Some(m.content.len()),
                source: Some("get_memory_tool".to_string()),
                ..Default::default()
            };
            experience.outcome = ExperienceOutcome::success();
            experience.tags = vec!["memory".to_string(), m.memory_type.to_string()];
            if let Err(e) = experience.commit() {
                tracing::warn!("Experience already committed: {}", e);
            }
            let memory_from_exp = MemoryCard::from_experience(&experience);
            queries::insert_memory(&conn, &memory_from_exp)?;

            Ok(ToolOutput::success(serde_json::json!({
                "found": true,
                "memory": {
                    "id": m.id.to_string(),
                    "content": m.content,
                    "memory_type": m.memory_type.to_string(),
                    "layer": m.layer.to_string(),
                    "confidence": m.confidence,
                    "importance": m.importance,
                    "created_at": m.created_at.to_rfc3339(),
                    "accessed_at": m.accessed_at.to_rfc3339()
                },
                "observation_id": observation.id.to_string(),
                "experience_id": experience.id.to_string()
            })))
        }
        None => Ok(ToolOutput::success(serde_json::json!({
            "found": false,
            "memory": serde_json::Value::Null
        }))),
    }
}

/// Execute list memories tool
/// Per Architecture §6.3: Uses MemoryRetrieval service
pub async fn execute_list_memories(
    input: ListMemoriesInput,
    database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(20);

    let working_items = memory_retrieval.get_context(limit).await;
    let working_count = working_items.len();

    let conn = database.connection()?;
    let db_memories = queries::search_memory(&conn, "", limit as usize)?;
    let db_ids: std::collections::HashSet<_> = db_memories.iter().map(|m| m.id).collect();
    let working_ids: std::collections::HashSet<_> = working_items.iter().map(|m| m.id).collect();

    let unique_db_memories: Vec<MemoryCard> = db_memories
        .into_iter()
        .filter(|m| !working_ids.contains(&m.id))
        .collect();

    let mut result: Vec<serde_json::Value> = working_items
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.to_string(),
                "content": m.content,
                "memory_type": m.memory_type.to_string(),
                "layer": m.layer.to_string(),
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339(),
                "accessed_at": m.accessed_at.to_rfc3339(),
                "source": "working_memory"
            })
        })
        .collect();

    for m in unique_db_memories {
        result.push(serde_json::json!({
            "id": m.id.to_string(),
            "content": m.content,
            "memory_type": m.memory_type.to_string(),
            "layer": m.layer.to_string(),
            "confidence": m.confidence,
            "importance": m.importance,
            "created_at": m.created_at.to_rfc3339(),
            "accessed_at": m.last_accessed.unwrap_or(m.created_at).to_rfc3339(),
            "source": "database"
        }));
    }

    Ok(ToolOutput::success(serde_json::json!({
        "memories": result,
        "count": result.len(),
        "working_count": working_count,
        "database_count": db_ids.len()
    })))
}

/// Execute archive memory tool
pub async fn execute_archive_memory(
    input: ArchiveMemoryInput,
    archived: bool,
) -> Result<ToolOutput> {
    Ok(ToolOutput::success(serde_json::json!({
        "success": archived,
        "memory_id": input.memory_id,
        "archived": archived,
    })))
}

/// Execute link memories tool
pub async fn execute_link_memories(input: LinkMemoriesInput) -> Result<ToolOutput> {
    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "from_id": input.from_id,
        "to_id": input.to_id,
        "relationship": "related",
    })))
}

/// Execute delete memory by ID tool
/// Requires explicit user confirmation (hard delete)
pub async fn execute_delete_memory(input: DeleteMemoryInput, deleted: bool) -> Result<ToolOutput> {
    Ok(ToolOutput::success(serde_json::json!({
        "success": deleted,
        "memory_id": input.memory_id,
        "deleted": deleted,
    })))
}
