// src/tools/experience/mod.rs
// Experience-related MCP tools

use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::queries;
use crate::database::sqlite::SqliteDatabase;
use crate::experience::coordinator::ExperienceCoordinator;
use crate::experience::types::{Experience, ExperienceOutcome, ExperienceType, OutcomeKind};
use crate::experience::worker_manager::WorkerManager;
use crate::bridge::tools::ToolOutput;

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
    pub const GET_WORKER_STATS: &str = "get_worker_stats";
    pub const GET_WORKER_COUNT: &str = "get_worker_count";

    pub fn all() -> Vec<crate::bridge::mcp::McpTool> {
        vec![
            crate::bridge::mcp::McpTool {
                name: RECORD_EXPERIENCE.to_string(),
                description: "Record a new experience from an action or observation".to_string(),
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
                description: "Get statistics about recorded experiences".to_string(),
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
                description: "List recent experiences".to_string(),
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
                description: "Get a specific experience by ID".to_string(),
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
                name: GET_WORKER_STATS.to_string(),
                description: "Get background worker statistics for observers".to_string(),
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
                description: "Get the number of active background workers".to_string(),
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

/// Execute get worker count tool
pub async fn execute_get_worker_count(worker_manager: &Arc<WorkerManager>) -> Result<ToolOutput> {
    let count = worker_manager.worker_count().await;

    Ok(ToolOutput::success(serde_json::json!({
        "worker_count": count
    })))
}
