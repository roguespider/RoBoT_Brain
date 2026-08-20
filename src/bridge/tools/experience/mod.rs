// src/tools/experience/mod.rs
// Experience-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::bridge::tools::ToolOutput;
use crate::database::models::MemoryCard;
use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::types::{Experience, ExperienceOutcome, ExperienceType, OutcomeKind};
use crate::experience::worker_manager::WorkerManager;

/// Tool: Record an experience
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RecordExperienceInput {
    pub title: String,
    pub description: String,
    pub experience_type: String,
    pub outcome: OutcomeKind,
    /// JSON-encoded context as a string (e.g., "{\"key\": \"value\"}")
    pub context: Option<String>,
    /// Caller-provided confidence (0.0–1.0). Defaults to 0.5.
    pub confidence: Option<f32>,
    /// Caller-provided importance (0.0–1.0). Defaults to 0.5.
    pub importance: Option<f32>,
    /// Tags for categorization.
    pub tags: Option<Vec<String>>,
}

/// Tool: Get experience statistics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetExperienceStatsInput {
    pub period: Option<String>,
}

/// Tool: List recent experiences
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct ListExperiencesInput {
    pub experience_type: Option<String>,
    pub limit: Option<usize>,
}

/// Tool: Get an experience by ID
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetExperienceInput {
    pub id: String,
}

/// Tool: Add evidence to an experience
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AddEvidenceToExperienceInput {
    /// Experience UUID
    pub experience_id: String,
    /// Evidence UUID to add
    pub evidence_id: String,
}

/// Tool: Archive an experience
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ArchiveExperienceInput {
    /// Experience UUID to archive
    pub experience_id: String,
}

// ============================================================================
// Background Worker Tools
// ============================================================================

/// Tool: Get worker statistics
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct GetWorkerStatsInput {
    pub observer_name: Option<String>,
}

/// Experience tool definitions
pub mod definitions {
    pub const RECORD_EXPERIENCE: &str = "record_experience";
    pub const GET_EXPERIENCE_STATS: &str = "get_experience_stats";
    pub const LIST_EXPERIENCES: &str = "list_experiences";
    pub const GET_EXPERIENCE: &str = "get_experience";
    pub const ADD_EVIDENCE_TO_EXPERIENCE: &str = "add_evidence_to_experience";
    pub const ARCHIVE_EXPERIENCE: &str = "archive_experience";
    pub const GET_WORKER_STATS: &str = "get_worker_stats";
    pub const GET_WORKER_COUNT: &str = "get_worker_count";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        macro_rules! desc {
            ($s:expr) => {
                format!("[WORKFLOW: get_workflow + search_memory first] {}", $s)
            };
        }
        vec![
            crate::bridge::mcp::McpTool {
                name: RECORD_EXPERIENCE.to_string(),
                description: desc!("Record a new experience from an action or observation"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Brief title for the experience"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed description of what happened"
                        },
                        "experience_type": {
                            "type": "string",
                            "description": "Type of experience",
                            "enum": ["tool_execution", "memory_lookup", "memory_store", "workflow", "planning", "exploration", "hypothesis", "reflection", "learning", "conversation", "user_feedback", "error", "system"]
                        },
                        "outcome": {
                            "type": "string",
                            "description": "Outcome of the experience",
                            "enum": ["success", "failure", "partial", "timeout", "interrupted"]
                        },
                        "context": {
                            "type": "string",
                            "description": "JSON-encoded context information (e.g., '{\"key\": \"value\"}')"
                        }
                    },
                    "required": ["title", "description", "experience_type", "outcome"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_EXPERIENCE_STATS.to_string(),
                description: desc!("Get statistics about recorded experiences"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "period": {
                            "type": "string",
                            "description": "Time period for stats: day, week, month, all",
                            "enum": ["day", "week", "month", "all"]
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: LIST_EXPERIENCES.to_string(),
                description: desc!("List recent experiences"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "experience_type": {
                            "type": "string",
                            "description": "Filter by experience type"
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
                name: GET_EXPERIENCE.to_string(),
                description: desc!("Get a specific experience by ID"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Experience UUID"
                        }
                    },
                    "required": ["id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ADD_EVIDENCE_TO_EXPERIENCE.to_string(),
                description: desc!("Add evidence to a recorded experience"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "experience_id": {
                            "type": "string",
                            "description": "Experience UUID"
                        },
                        "evidence_id": {
                            "type": "string",
                            "description": "Evidence UUID to add to this experience"
                        }
                    },
                    "required": ["experience_id", "evidence_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: ARCHIVE_EXPERIENCE.to_string(),
                description: desc!("Archive an experience (soft-delete, not destroy)"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "experience_id": {
                            "type": "string",
                            "description": "Experience UUID to archive"
                        }
                    },
                    "required": ["experience_id"]
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_WORKER_STATS.to_string(),
                description: desc!("Get background worker statistics for observers"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "observer_name": {
                            "type": "string",
                            "description": "Filter stats by observer name (optional)"
                        }
                    }
                }),
            },
            crate::bridge::mcp::McpTool {
                name: GET_WORKER_COUNT.to_string(),
                description: desc!("Get the number of active background workers"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        ]
    }
}

fn parse_experience_type(s: &str) -> ExperienceType {
    match s.to_lowercase().as_str() {
        "tool_execution" | "tool" => ExperienceType::ToolExecution,
        "memory_lookup" | "lookup" => ExperienceType::MemoryLookup,
        "memory_store" | "store" => ExperienceType::MemoryStore,
        "workflow" => ExperienceType::Workflow,
        "planning" => ExperienceType::Planning,
        "exploration" => ExperienceType::Exploration,
        "hypothesis" => ExperienceType::Hypothesis,
        "reflection" => ExperienceType::Reflection,
        "learning" => ExperienceType::Learning,
        "conversation" => ExperienceType::Conversation,
        "user_feedback" | "feedback" => ExperienceType::UserFeedback,
        "error" => ExperienceType::Error,
        "system" => ExperienceType::System,
        _ => ExperienceType::Custom(s.to_string()),
    }
}

fn outcome_kind_to_experience_outcome(kind: OutcomeKind) -> ExperienceOutcome {
    match kind {
        OutcomeKind::Success => ExperienceOutcome::success(),
        OutcomeKind::Failure => ExperienceOutcome::failure("Recorded via MCP tool"),
        OutcomeKind::Partial => ExperienceOutcome::partial("Partial success"),
        OutcomeKind::Timeout => ExperienceOutcome::timeout(),
        OutcomeKind::Interrupted => ExperienceOutcome::interrupted(),
    }
}

/// Execute record experience tool
pub async fn execute_record_experience(
    input: RecordExperienceInput,
    coordinator: &Arc<ExperienceCoordinator>,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    // Validate context JSON if provided
    if let Some(ref ctx_str) = input.context {
        let validated_context: serde_json::Value = serde_json::from_str(ctx_str)
            .map_err(|e| anyhow::anyhow!("Invalid JSON in context: {}", e))?;
        // Validate that context has expected structure
        if !validated_context.is_object() && !validated_context.is_array() {
            return Err(anyhow::anyhow!("Context must be a JSON object or array"));
        }
    }

    // Create experience with observation origins (Architecture §07 invariant)
    let mut experience = Experience::new(
        input.title.clone(),
        input.description.clone(),
        parse_experience_type(&input.experience_type),
        vec![], // observation_ids populated by observer
    );

    // Set outcome
    experience.outcome = outcome_kind_to_experience_outcome(input.outcome);

    // Set caller-provided confidence and tags
    if let Some(c) = input.confidence {
        experience.confidence = c.clamp(0.0, 1.0);
    }
    if let Some(tags) = input.tags {
        experience.tags = tags;
    }
    // Pre-score the experience so that knowledge promotion (which requires
    // score >= 0.8) can fire for high-confidence successful experiences
    // recorded via MCP. The coordinator's process() will re-score, but
    // having the initial score set ensures the threshold check passes.
    if let Some(importance) = input.importance {
        let initial_score = crate::experience::types::ExperienceScore {
            importance: importance.clamp(0.0, 1.0),
            confidence: experience.confidence,
            novelty: 0.0,
            reliability: experience.confidence,
        };
        experience.score = Some(initial_score);
    }

    // Process through coordinator for scoring and event emission
    // This publishes:
    // 1. Scored event (for WorkerManager and observers)
    // 2. ExperienceRecorded event (for EventSubscriber - triggers Reflection → Hypothesis → Knowledge → Reputation)
    let processed = coordinator.process(experience.clone());

    // Store in database
    let conn = database.connection()?;
    let memory = crate::database::models::MemoryCard::from_experience(&processed);
    queries::insert_memory(&conn, &memory)?;

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "message": "Experience recorded successfully",
        "id": processed.id.to_string(),
        "title": processed.title
    })))
}

/// Execute get experience stats tool
pub async fn execute_get_experience_stats(
    _: GetExperienceStatsInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let conn = database.connection()?;
    let memories = queries::search_memory(&conn, "Experience:", 1000)?;

    let total = memories.len();

    // Count by type (simplified - counts all experiences)
    let by_type = serde_json::json!({
        "total": total
    });

    // Count by outcome
    let mut success = 0;
    let mut failure = 0;
    for m in &memories {
        if m.content.contains("Success") || m.content.contains("success") {
            success += 1;
        } else {
            failure += 1;
        }
    }

    let by_outcome = serde_json::json!({
        "success": success,
        "failure": failure
    });

    Ok(ToolOutput::success(serde_json::json!({
        "stats": {
            "total": total,
            "by_type": by_type,
            "by_outcome": by_outcome
        },
        "total": total,
        "by_type": by_type,
        "by_outcome": by_outcome
    })))
}

/// Execute list experiences tool
pub async fn execute_list_experiences(
    input: ListExperiencesInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let limit = input.limit.unwrap_or(20);
    let conn = database.connection()?;
    let memories = queries::search_memory(&conn, "Experience:", limit)?;

    let experiences: Vec<serde_json::Value> = memories
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id.to_string(),
                "content": m.content,
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339()
            })
        })
        .collect();

    Ok(ToolOutput::success(serde_json::json!({
        "experiences": experiences,
        "count": experiences.len()
    })))
}

/// Execute get experience tool
pub async fn execute_get_experience(
    input: GetExperienceInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let uuid = Uuid::parse_str(&input.id).map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;

    let conn = database.connection()?;
    let memory = queries::get_memory(&conn, uuid)?;

    match memory {
        Some(m) => Ok(ToolOutput::success(serde_json::json!({
            "id": m.id.to_string(),
            "found": true,
            "experience": {
                "id": m.id.to_string(),
                "content": m.content,
                "confidence": m.confidence,
                "importance": m.importance,
                "created_at": m.created_at.to_rfc3339(),
                "updated_at": m.updated_at.to_rfc3339()
            }
        }))),
        None => Ok(ToolOutput::success(serde_json::json!({
            "id": null,
            "found": false,
            "experience": serde_json::Value::Null
        }))),
    }
}

// ============================================================================
// Background Worker Tools Implementation
// ============================================================================

/// Execute get worker stats tool
pub async fn execute_get_worker_stats(
    input: GetWorkerStatsInput,
    worker_manager: &Arc<WorkerManager>,
) -> Result<ToolOutput> {
    let stats =
        if let Some(observer_name) = &input.observer_name {
            worker_manager.get_observer_stats(observer_name).await
            .map(|s| {
                serde_json::json!([{
                    "observer_name": s.observer_name,
                    "jobs_processed": s.jobs_processed.load(std::sync::atomic::Ordering::SeqCst),
                    "jobs_failed": s.jobs_failed.load(std::sync::atomic::Ordering::SeqCst),
                    "jobs_retried": s.jobs_retried.load(std::sync::atomic::Ordering::SeqCst),
                }])
            })
            .unwrap_or_else(|| serde_json::json!([]))
        } else {
            let all_stats = worker_manager.get_stats().await;
            serde_json::json!(all_stats.iter().map(|s| {
            serde_json::json!({
                "observer_name": s.observer_name,
                "jobs_processed": s.jobs_processed.load(std::sync::atomic::Ordering::SeqCst),
                "jobs_failed": s.jobs_failed.load(std::sync::atomic::Ordering::SeqCst),
                "jobs_retried": s.jobs_retried.load(std::sync::atomic::Ordering::SeqCst),
            })
        }).collect::<Vec<_>>())
        };

    Ok(ToolOutput::success(serde_json::json!({
        "stats": stats,
        "worker_count": worker_manager.worker_count().await
    })))
}

/// Execute add evidence to experience tool
pub async fn execute_add_evidence_to_experience(
    input: AddEvidenceToExperienceInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let experience_uuid = Uuid::parse_str(&input.experience_id)
        .map_err(|e| anyhow::anyhow!("Invalid experience UUID: {}", e))?;
    let evidence_uuid = Uuid::parse_str(&input.evidence_id)
        .map_err(|e| anyhow::anyhow!("Invalid evidence UUID: {}", e))?;

    let conn = database.connection()?;
    let memory = queries::get_memory(&conn, experience_uuid)?
        .ok_or_else(|| anyhow::anyhow!("Experience not found: {}", experience_uuid))?;

    // Convert MemoryCard back to Experience to call add_evidence
    let experience = MemoryCard::into_experience(memory.clone());

    // Validate the experience is not already committed (Architecture §07 invariant)
    if experience.committed {
        return Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "message": "Cannot add evidence to a committed experience (immutable)"
        })));
    }

    // Call the add_evidence method on Experience
    let mut mutable_experience = experience.clone();
    mutable_experience.add_evidence(evidence_uuid);

    // Upsert the memory card with the modified experience
    let updated_memory = MemoryCard::from_experience(&mutable_experience);
    queries::insert_memory(&conn, &updated_memory)?;

    Ok(ToolOutput::success(serde_json::json!({
        "success": true,
        "message": "Evidence added to experience",
        "experience_id": experience_uuid.to_string(),
        "evidence_id": evidence_uuid.to_string(),
        "evidence_count": mutable_experience.evidence_count
    })))
}

/// Execute archive experience tool
pub async fn execute_archive_experience(
    input: ArchiveExperienceInput,
    database: &Arc<SqliteDatabase>,
) -> Result<ToolOutput> {
    let experience_uuid = Uuid::parse_str(&input.experience_id)
        .map_err(|e| anyhow::anyhow!("Invalid experience UUID: {}", e))?;

    let conn = database.connection()?;
    let memory = queries::get_memory(&conn, experience_uuid)?
        .ok_or_else(|| anyhow::anyhow!("Experience not found: {}", experience_uuid))?;

    let experience = MemoryCard::into_experience(memory.clone());

    // Call the archive method on Experience
    let mut mutable_experience = experience.clone();
    match mutable_experience.archive() {
        Ok(()) => {
            // Upsert the memory card with the archived experience
            let archived_memory = MemoryCard::from_experience(&mutable_experience);
            queries::insert_memory(&conn, &archived_memory)?;

            Ok(ToolOutput::success(serde_json::json!({
                "success": true,
                "message": "Experience archived successfully",
                "experience_id": experience_uuid.to_string()
            })))
        }
        Err(e) => Ok(ToolOutput::success(serde_json::json!({
            "success": false,
            "message": e
        }))),
    }
}

/// Execute get worker count tool
pub async fn execute_get_worker_count(worker_manager: &Arc<WorkerManager>) -> Result<ToolOutput> {
    let count = worker_manager.worker_count().await;

    Ok(ToolOutput::success(serde_json::json!({
        "worker_count": count
    })))
}
