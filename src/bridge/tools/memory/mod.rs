// src/tools/memory/mod.rs
// Memory-related MCP tools
// Per Architecture §07: Every experience originates from observations

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::{MemoryCard, MemoryType, Observation};
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::types::{Experience, ExperienceContext, ExperienceOutcome, ExperienceType};
use crate::memory::types::{MemoryItem, MemoryLayer};
use crate::memory::{MemoryRetrieval, WorkingMemory};
use crate::bridge::tools::ToolOutput;

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

/// Memory tool definitions
pub mod definitions {
    pub const STORE_MEMORY: &str = "store_memory";
    pub const SEARCH_MEMORY: &str = "search_memory";
    pub const GET_MEMORY: &str = "get_memory";
    pub const LIST_MEMORIES: &str = "list_memories";
    pub const STORE_EMBEDDING: &str = "store_embedding";
    pub const GET_EMBEDDING: &str = "get_embedding";
    pub const SEARCH_SIMILAR: &str = "search_similar";
    pub const LIST_EMBEDDINGS: &str = "list_embeddings";
    pub const DELETE_EMBEDDING: &str = "delete_embedding";
    pub const GET_EMBEDDING_STATS: &str = "get_embedding_stats";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: STORE_MEMORY.to_string(),
                description: "Store a new memory in the knowledge base".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The content to store"
                        },
                        "memory_type": {
                            "type": "string",
                            "description": "Type of memory: note, fact, task, file, conversation, code, decision, event",
                            "enum": ["note", "fact", "task", "file", "conversation", "code", "decision", "event"]
                        },
                        "confidence": {
                            "type": "number",
                            "description": "Confidence level (0.0 - 1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "importance": {
                            "type": "number",
                            "description": "Importance level (0.0 - 1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0
                        },
                        "tags": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional tags for categorization"
                        }
                    },
                    "required": ["content", "memory_type"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: SEARCH_MEMORY.to_string(),
                description: "Search memories by content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 10
                        }
                    },
                    "required": ["query"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_MEMORY.to_string(),
                description: "Get a specific memory by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Memory UUID"
                        }
                    },
                    "required": ["id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_MEMORIES.to_string(),
                description: "List recent memories".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_type": {
                            "type": "string",
                            "description": "Filter by memory type"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 20
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: STORE_EMBEDDING.to_string(),
                description: "Store a vector embedding for semantic memory search".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "The memory UUID to associate with this embedding"
                        },
                        "embedding": {
                            "type": "array",
                            "items": { "type": "number" },
                            "description": "The vector embedding as an array of floats"
                        },
                        "model": {
                            "type": "string",
                            "description": "The model used to generate the embedding",
                            "default": "default"
                        }
                    },
                    "required": ["memory_id", "embedding"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_EMBEDDING.to_string(),
                description: "Get an embedding by memory ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "The memory UUID"
                        }
                    },
                    "required": ["memory_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: SEARCH_SIMILAR.to_string(),
                description: "Search for similar memories using vector similarity".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query_embedding": {
                            "type": "array",
                            "items": { "type": "number" },
                            "description": "The query vector as an array of floats"
                        },
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 5
                        },
                        "min_similarity": {
                            "type": "number",
                            "description": "Minimum cosine similarity threshold (0.0 - 1.0)",
                            "minimum": 0.0,
                            "maximum": 1.0,
                            "default": 0.5
                        }
                    },
                    "required": ["query_embedding"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_EMBEDDINGS.to_string(),
                description: "List all memory embeddings".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": {
                            "type": "number",
                            "description": "Maximum number of results",
                            "default": 100
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: DELETE_EMBEDDING.to_string(),
                description: "Delete an embedding by memory ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "The memory UUID"
                        }
                    },
                    "required": ["memory_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_EMBEDDING_STATS.to_string(),
                description: "Get vector index statistics".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }
}

fn parse_memory_type(s: &str) -> MemoryType {
    match s.to_lowercase().as_str() {
        "fact" => MemoryType::Fact,
        "task" => MemoryType::Task,
        "file" => MemoryType::File,
        "conversation" => MemoryType::Conversation,
        "code" => MemoryType::Code,
        "decision" => MemoryType::Decision,
        "event" => MemoryType::Event,
        "encounter" => MemoryType::Encounter,
        "experience" => MemoryType::Experience,
        _ => MemoryType::Note,
    }
}

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

/// Convert database MemoryType to memory module MemoryType
fn convert_memory_type_to_memory(dt: MemoryType) -> crate::memory::types::MemoryType {
    match dt {
        MemoryType::Note => crate::memory::types::MemoryType::Experience,
        MemoryType::Fact => crate::memory::types::MemoryType::Knowledge,
        MemoryType::Task => crate::memory::types::MemoryType::Skill,
        MemoryType::File => crate::memory::types::MemoryType::Workflow,
        MemoryType::Conversation => crate::memory::types::MemoryType::Context,
        MemoryType::Code => crate::memory::types::MemoryType::Skill,
        MemoryType::Decision => crate::memory::types::MemoryType::Experience,
        MemoryType::Event => crate::memory::types::MemoryType::Observation,
        MemoryType::Encounter => crate::memory::types::MemoryType::Observation,
        MemoryType::Experience => crate::memory::types::MemoryType::Experience,
    }
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

// ============================================================================
// VECTOR INDEX TOOLS (Embedding Operations)
// ============================================================================

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
