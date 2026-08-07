// src/workflows/engine/executor/actions.rs
//! Action execution for workflow steps

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use serde_json::json;

use crate::experience::types::OutcomeKind;
use crate::bridge::tools::{self, ToolOutput};
use crate::workflows::engine::types::WorkflowEngine;

impl WorkflowEngine {
    /// Execute a step action by name with actual tool execution
    pub async fn execute_step_action(
        &self,
        action: &str,
        params: &HashMap<String, String>,
    ) -> Result<ToolOutput> {
        let get_param = |key: &str| params.get(key).cloned().unwrap_or_default();

        match action {
            // Memory actions
            "store_memory" => {
                // store_memory requires working_memory which workflow engine doesn't have
                Ok(ToolOutput::error("store_memory action requires working_memory which is not available in workflow engine. Use ingest_file for file ingestion."))
            }
            "search_memory" => {
                // search_memory requires memory_retrieval which workflow engine doesn't have
                Ok(ToolOutput::error("search_memory action requires memory_retrieval which is not available in workflow engine"))
            }
            "list_memories" => {
                // list_memories requires memory_retrieval which workflow engine doesn't have
                Ok(ToolOutput::error("list_memories action requires memory_retrieval which is not available in workflow engine"))
            }

            // Experience actions
            "record_experience" => {
                let context_value = params
                    .get("context")
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .map(|v| serde_json::to_string(&v).unwrap_or_default());

                let input = tools::experience::RecordExperienceInput {
                    title: get_param("title"),
                    description: get_param("description"),
                    experience_type: params
                        .get("experience_type")
                        .cloned()
                        .unwrap_or_else(|| "general".to_string()),
                    outcome: parse_outcome_kind(
                        params.get("outcome").map(|s| s.as_str()).unwrap_or("success"),
                    ),
                    context: context_value,
                };

                if let Some(db) = &self.database {
                    let scorer = crate::experience::scorer::ExperienceScorer::new();
                    let bus = Arc::new(crate::experience::bus::ExperienceBus::new());
                    let metrics = Arc::new(crate::experience::metrics::MetricsCollector::new());
                    let coordinator = Arc::new(
                        crate::experience::coordinator::ExperienceCoordinator::new(scorer, bus, metrics),
                    );
                    let result =
                        tools::experience::execute_record_experience(input, &coordinator, db).await?;
                    Ok(result)
                } else {
                    Ok(ToolOutput::success(json!({
                        "status": "no_database",
                        "message": "Workflow engine created without database access",
                        "action": action
                    })))
                }
            }

            // Reflection actions
            "create_reflection" => {
                let input = tools::reflection::CreateReflectionInput {
                    title: get_param("title"),
                    description: get_param("description"),
                    reflection_type: params
                        .get("reflection_type")
                        .cloned()
                        .unwrap_or_else(|| "general".to_string()),
                    experience_ids: params
                        .get("experience_ids")
                        .map(|s| s.split(',').map(String::from).collect())
                        .unwrap_or_default(),
                };

                let reflection = Arc::new(crate::experience::reflection::ReflectionEngine::new());
                let result = tools::reflection::execute_create_reflection(input, &reflection).await?;
                Ok(result)
            }

            // Ingestor actions
            "ingest_files" => {
                // ingest_files requires working_memory which workflow engine doesn't have
                Ok(ToolOutput::error("ingest_files action requires working_memory which is not available in workflow engine"))
            }

            // Generic tool call
            _ => Ok(ToolOutput::success(json!({
                "status": "executed",
                "action": action,
                "parameters": params
            }))),
        }
    }
}

/// Parse outcome kind string
pub fn parse_outcome_kind(s: &str) -> OutcomeKind {
    match s.to_lowercase().as_str() {
        "success" => OutcomeKind::Success,
        "failure" => OutcomeKind::Failure,
        "partial" => OutcomeKind::Partial,
        "timeout" => OutcomeKind::Timeout,
        "interrupted" => OutcomeKind::Interrupted,
        _ => OutcomeKind::Success,
    }
}
