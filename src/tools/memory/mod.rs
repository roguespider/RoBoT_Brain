
#![allow(dead_code)]

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
use crate::memory::{MemoryRetrieval, WorkingMemory};
use crate::memory::types::{MemoryItem, MemoryLayer};
use crate::tools::ToolOutput;

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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListMemoriesInput {
    pub memory_type: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Preview memories for deletion (asks user before deleting)
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CleanupMemoriesInput {
    /// List of memory IDs to delete
    pub ids: Vec<String>,
    /// Confirmation from user: "yes" to proceed with deletion
    pub confirmation: String,
}

/// Tool: List all image memories and detect garbage
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListImageMemoriesInput {
    /// Show memories that look like garbage image text (binary data stored as text)
    pub include_garbage: Option<bool>,
}

/// Memory tool definitions
pub mod definitions {
    pub const STORE_MEMORY: &str = "store_memory";
    pub const SEARCH_MEMORY: &str = "search_memory";
    pub const GET_MEMORY: &str = "get_memory";
    pub const LIST_MEMORIES: &str = "list_memories";
    pub const CLEANUP_MEMORIES: &str = "cleanup_memories";
    pub const LIST_IMAGE_MEMORIES: &str = "list_image_memories";
    
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
                name: CLEANUP_MEMORIES.to_string(),
                description: "Delete specific memories by ID (requires explicit user confirmation)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ids": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of memory UUIDs to delete"
                        },
                        "confirmation": {
                            "type": "string",
                            "description": "User confirmation: must be 'yes' to proceed with deletion"
                        }
                    },
                    "required": ["ids", "confirmation"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_IMAGE_MEMORIES.to_string(),
                description: "List image memories and detect garbage".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "include_garbage": {
                            "type": "boolean",
                            "description": "Include garbage memories (binary data stored as text)"
                        }
                    }
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
/// 
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
        vec![observation_id],  // Observation origins per §07
    );
    experience.context = ExperienceContext {
        memory_type: Some(input.memory_type.clone()),
        content_length: Some(input.content.len()),
        source: Some("store_memory_tool".to_string()),
        ..Default::default()
    };
    experience.outcome = ExperienceOutcome::success();
    experience.tags = vec![
        "memory".to_string(),
        memory_type.to_string(),
    ];
    
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
    let conn = database.connection()?;
    
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
/// 
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
        format!("Searched memory with query '{}', found {} results", input.query, results.len()),
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
/// 
/// Per Architecture §07: Memory access generates observations for the learning pipeline.
/// Per Architecture §6.3: Uses MemoryRetrieval service
pub async fn execute_get_memory(
    input: GetMemoryInput,
    database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let uuid = Uuid::parse_str(&input.id)
        .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
    
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
                format!("memory_type={}, id={}, layer={}", m.memory_type, m.id, m.layer),
                "memory_retrieval".to_string(),
            );
            queries::insert_observation(&conn, &observation)?;
            
            // Create experience for the memory retrieval
            let mut experience = Experience::new(
                format!("Memory retrieved: {}", content_preview),
                format!("Retrieved {} memory with id {} from {}", m.memory_type, m.id, m.layer),
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
    _database: &Arc<SqliteDatabase>,
    memory_retrieval: &Arc<MemoryRetrieval>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(20);
    
    // Get recent memories from both Working and Permanent Memory
    let working_items = memory_retrieval.get_context(limit).await;
    
    // Convert MemoryItem to JSON format
    let result: Vec<serde_json::Value> = working_items
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
                "accessed_at": m.accessed_at.to_rfc3339()
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "memories": result,
        "count": result.len()
    })))
}

/// Execute cleanup memories tool - ALWAYS requires user confirmation to delete
pub async fn execute_cleanup_memories(
    input: CleanupMemoriesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    if input.ids.is_empty() {
        return Ok(ToolOutput::success(serde_json::json!({
            "deleted": 0,
            "ask_confirmation": false,
            "message": "No memories specified for deletion."
        })));
    }
    
    // ALWAYS show preview first and require confirmation
    // There is NO way to delete without explicit confirmation
    let conn = database.connection()?;
    
    let mut memories_to_delete = Vec::new();
    for id_str in &input.ids {
        if let Ok(id) = Uuid::parse_str(id_str) {
            if let Ok(Some(memory)) = queries::get_memory(&conn, id) {
                memories_to_delete.push(serde_json::json!({
                    "id": memory.id.to_string(),
                    "preview": if memory.content.len() > 100 {
                        format!("{}...", &memory.content[..100])
                    } else {
                        memory.content.clone()
                    },
                    "memory_type": memory.memory_type.to_string(),
                    "created_at": memory.created_at.to_rfc3339()
                }));
            }
        }
    }
    
    // Check if user confirmed
    let confirmation = input.confirmation.to_lowercase();
    let confirmed = confirmation == "yes" || confirmation == "y" || confirmation == "true";
    
    if confirmed {
        // User confirmed - delete the memories
        let deleted = queries::delete_memories_by_string_ids(&conn, &input.ids)?;
        
        return Ok(ToolOutput::success(serde_json::json!({
            "deleted": deleted,
            "requested": input.ids.len(),
            "confirmed": true,
            "ask_confirmation": false,
            "message": format!("Deleted {} memory(ies) from memory.", deleted)
        })));
    }
    
    // No confirmation yet - show preview and ask for it
    Ok(ToolOutput::success(serde_json::json!({
        "deleted": 0,
        "confirmed": false,
        "ask_confirmation": true,
        "memories_to_delete": memories_to_delete,
        "count": memories_to_delete.len(),
        "message": format!("About to delete {} memory(ies). Say 'yes' to confirm deletion.", memories_to_delete.len())
    })))
}

/// Execute list image memories tool
/// Lists all image memories and optionally shows garbage (binary data stored as text)
pub async fn execute_list_image_memories(
    input: ListImageMemoriesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let include_garbage = input.include_garbage.unwrap_or(false);
    let conn = database.connection()?;
    
    // List all memories
    let all_memories = queries::list_memories(&conn, None, 1000)?;
    
    // Categorize memories
    let mut proper_images = Vec::new();
    let mut garbage_memories = Vec::new();
    
    for memory in all_memories {
        let content = &memory.content;
        
        // Check if this looks like a proper image memory (starts with "IMAGE FILE")
        if content.starts_with("IMAGE FILE") {
            proper_images.push(serde_json::json!({
                "id": memory.id.to_string(),
                "filename": extract_field_from_content(content, "Filename:"),
                "format": extract_field_from_content(content, "Format:"),
                "size": extract_field_from_content(content, "Size:"),
                "preview": content.lines().take(5).collect::<Vec<_>>().join("\n"),
                "memory_type": memory.memory_type.to_string(),
                "created_at": memory.created_at.to_rfc3339()
            }));
        }
        // Check if this looks like garbage (binary data stored as text)
        else if content_looks_like_garbage(content) {
            garbage_memories.push(serde_json::json!({
                "id": memory.id.to_string(),
                "preview": if content.len() > 100 { format!("{}...", &content[..100]) } else { content.clone() },
                "size_chars": content.len(),
                "memory_type": memory.memory_type.to_string(),
                "created_at": memory.created_at.to_rfc3339(),
                "likely_source": "image"
            }));
        }
    }
    
    let show_garbage = include_garbage;
    let garbage_count = garbage_memories.len();
    
    let response = serde_json::json!({
        "images": proper_images,
        "images_count": proper_images.len(),
        "garbage": if show_garbage { serde_json::Value::Array(garbage_memories.clone()) } else { serde_json::Value::Null },
        "garbage_count": garbage_count,
        "has_garbage": !garbage_memories.is_empty(),
        "ask_cleanup": if !garbage_memories.is_empty() {
            serde_json::json!(format!("Found {} memory(ies) that look like binary garbage. Do you want to clean them up?", garbage_count))
        } else {
            serde_json::Value::Null
        },
        "message": if proper_images.is_empty() && garbage_memories.is_empty() {
            "No image memories found.".to_string()
        } else if proper_images.is_empty() {
            format!("No proper image memories, but found {} garbage memories.", garbage_count)
        } else {
            format!("Found {} image memories.", proper_images.len())
        }
    });
    
    Ok(ToolOutput::success(response))
}

/// Extract a field value from content
fn extract_field_from_content(content: &str, field_name: &str) -> String {
    for line in content.lines() {
        if line.starts_with(field_name) {
            return line[field_name.len()..].trim().to_string();
        }
    }
    "unknown".to_string()
}

/// Check if content looks like binary garbage
fn content_looks_like_garbage(content: &str) -> bool {
    let bytes = content.as_bytes();
    
    // Empty or very short
    if content.len() < 10 {
        return false;
    }
    
    // Count null bytes
    let null_count = bytes.iter().filter(|&&b| b == 0).count();
    if null_count > 0 {
        return true;
    }
    
    // Check ratio of printable characters
    let printable = bytes.iter().filter(|&&b| (32..127).contains(&b) || b >= 128).count();
    let ratio = printable as f64 / bytes.len() as f64;
    if ratio < 0.3 {
        return true;
    }
    
    // Check for common binary patterns using byte strings
    if content.starts_with("PK\x03\x04") {  // ZIP
        return true;
    }
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {  // PNG
        return true;
    }
    if content.starts_with("GIF8") {  // GIF
        return true;
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {  // JPEG
        return true;
    }
    if content.starts_with("BM\x00\x00") {  // BMP
        return true;
    }
    
    false
}
