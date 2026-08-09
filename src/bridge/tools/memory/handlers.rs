//! Memory CRUD tool handlers.
//!
//! Each handler wires the MCP request through the cognitive pipeline
//! described in the architecture: an observation is recorded, an experience
//! is derived from it, and the resulting memory is cached in Working Memory
//! (Architecture §6.3) and checkpointed to the database for persistence.

use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::database::models::{MemoryCard, Observation};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{
    Experience, ExperienceContext, ExperienceOutcome, ExperienceType,
};
use crate::memory::types::{MemoryItem, MemoryLayer};
use crate::memory::{MemoryRetrieval, WorkingMemory};

use super::helpers::{convert_memory_type_to_memory, parse_memory_type};
use super::types::{
    ArchiveMemoryInput, GetMemoryInput, LinkMemoriesInput, ListMemoriesInput, RankedSearchInput,
    SearchMemoryInput, StoreMemoryInput,
};

/// Execute store memory tool
/// Per Architecture §07: Every experience originates from observations
/// Per Architecture §1: Memory is a component. Experience is the source of learning.
/// Per Architecture §4: "Actions, observations, decisions, successes, failures,
///                      and discoveries should create experiences."
/// Per Architecture §6.3: Stores in Working Memory (fast, volatile, in-memory cache)
pub async fn execute_store_memory(
    input: StoreMemoryInput,
    database: &Arc<SqliteDatabase>,
    working_memory: &Arc<WorkingMemory>,
) -> Result<ToolOutput> {
    let conn = database.connection()?;

    let memory_type = parse_memory_type(&input.memory_type);

    // Step 1: Create an Observation (Per Architecture §07 invariant)
    // "Every experience originates from one or more observations"
    let content_preview = if input.content.len() > 100 {
        format!("{}...", &input.content[..100])
    } else {
        input.content.clone()
    };
    let observation = Observation::new(
        content_preview.clone(),
        format!("memory_type={}", input.memory_type),
        "memory_store".to_string(),
    );
    let observation_id = observation.id;

    // Step 2: Create an Experience with observation origin (Per Architecture §07)
    // "Experience answers: What happened, what did we learn, and what should change?"
    let mut experience = Experience::new(
        format!("Memory stored: {}", input.memory_type),
        format!("Stored {} memory: {}", input.memory_type, content_preview),
        ExperienceType::MemoryStore,
        vec![observation_id], // Observation origins per §07
    );
    experience.context = ExperienceContext {
        memory_type: Some(input.memory_type.clone()),
        content_length: Some(input.content.len()),
        source: Some("store_memory_tool".to_string()),
        ..Default::default()
    };
    experience.outcome = ExperienceOutcome::success();
    experience.tags = vec!["memory".to_string(), memory_type.to_string()];

    // Step 3: Create the MemoryItem for Working Memory cache (Architecture §6.3)
    let mut memory_item = MemoryItem::new(
        MemoryLayer::Working,
        convert_memory_type_to_memory(memory_type.clone()),
        input.content.clone(),
        "store_memory_tool".to_string(),
    );
    memory_item.confidence = input.confidence.unwrap_or(0.5);
    memory_item.importance = input.importance.unwrap_or(0.5);
    if let Some(tags) = input.tags {
        for tag in tags {
            memory_item.add_tag(tag);
        }
    }

    let memory_id = memory_item.id;
    let experience_id = experience.id;

    // Store in Working Memory cache (Architecture §6.3)
    // This is the PRIMARY storage - fast, in-memory
    working_memory.store(memory_item.clone()).await;

    // Also checkpoint to database for persistence
    // conn is already obtained at the beginning for precondition check

    // Store observation first (per Architecture §07: experiences originate from observations)
    queries::insert_observation(&conn, &observation)?;

    // Commit and store experience (commit returns Result<(), &'static str>)
    if let Err(e) = experience.commit() {
        tracing::warn!("Experience already committed: {}", e);
    }
    let memory_from_exp = MemoryCard::from_experience(&experience);
    queries::insert_memory(&conn, &memory_from_exp)?;

    // Also store the actual memory in database for recovery
    let memory_card: MemoryCard = memory_item.into();
    queries::insert_memory(&conn, &memory_card)?;

    tracing::info!(
        "Memory stored in Working Memory cache with observation and experience: memory_id={}, observation_id={}, experience_id={}",
        memory_id, observation_id, experience_id
    );

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "message": "Memory stored successfully in Working Memory cache with observation and experience",
        "id": memory_id.to_string(),
        "observation_id": observation_id.to_string(),
        "experience_id": experience_id.to_string(),
        "layer": "working",  // Per Architecture §6.3: Working Memory
        "note": "Per Architecture §9: Memory will be evaluated before promotion to Permanent layer"
    })))
}

/// Execute search memory tool
/// Per Architecture §07: Memory access generates observations for the learning pipeline.
/// Per Architecture §4: Memory retrieval is part of the event system.
/// Per Architecture §6.3: Uses MemoryRetrieval service (queries both Working and Permanent memory)
pub async fn execute_search_memory(
    input: SearchMemoryInput,
    database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(10);

    // Search using MemoryRetrieval service (Architecture §6.3)
    // This queries both Working Memory and Permanent Memory caches
    let results = memory_retrieval.retrieve(&input.query).await;

    // Take only the requested limit
    let results: Vec<_> = results.into_iter().take(limit).collect();

    // Create observation for memory lookup (Per Architecture §07)
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

    // Create experience for the memory lookup
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

/// Execute get memory tool
/// Per Architecture §07: Memory access generates observations for the learning pipeline.
/// Per Architecture §6.3: Uses MemoryRetrieval service
pub async fn execute_get_memory(
    input: GetMemoryInput,
    database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let uuid = Uuid::parse_str(&input.id).map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;

    // Try to get from Working Memory first, then Permanent Memory
    let working = memory_retrieval.working_memory().retrieve(&uuid).await;
    let permanent = memory_retrieval.permanent_memory().retrieve(&uuid).await;

    let memory_item = working.or(permanent);

    match memory_item {
        Some(m) => {
            let conn = database.connection()?;

            // Create observation for memory retrieval (Per Architecture §07)
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

            // Create experience for the memory retrieval
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

    // Get recent memories from both Working Memory cache and database
    let working_items = memory_retrieval.get_context(limit).await;
    let working_count = working_items.len();

    // Also query the database for memories not in working memory
    let conn = database.connection()?;
    let db_memories = queries::search_memory(&conn, "", limit as usize)?;
    let db_ids: std::collections::HashSet<_> = db_memories.iter().map(|m| m.id).collect();
    let working_ids: std::collections::HashSet<_> = working_items.iter().map(|m| m.id).collect();

    // Deduplicate by only including database memories not already in working memory
    let unique_db_memories: Vec<MemoryCard> = db_memories
        .into_iter()
        .filter(|m| !working_ids.contains(&m.id))
        .collect();

    // Convert MemoryItem to JSON format
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

    // Add database-only memories
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
