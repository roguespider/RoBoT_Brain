// src/workflows/engine/executor/experience.rs
//! Experience recording for workflow actions

use std::collections::HashMap;

use crate::bridge::tools::ToolOutput;
use crate::experience::types::OutcomeKind;
use crate::workflows::engine::SKIP_MEMORY_READ;
use crate::workflows::engine::types::WorkflowEngine;

use crate::workflows::engine::experience::{
    build_experience_description, map_action_to_experience_type,
};

impl WorkflowEngine {
    /// Check if action should skip memory read
    pub fn should_skip_memory_read(action: &str) -> bool {
        SKIP_MEMORY_READ.contains(&action)
    }

    /// Automatically read relevant memories before executing an action
    pub async fn read_memory_before_action(
        &self,
        action: &str,
        params: &HashMap<String, String>,
    ) -> Option<ToolOutput> {
        // Check skip list first (P4-002C)
        if Self::should_skip_memory_read(action) {
            tracing::trace!("[Memory] Skipping memory read for action '{}'", action);
            return None;
        }

        // Use memory_retrieval if available (P4-002A-3)
        if let Some(retrieval) = &self.memory_retrieval {
            // Build a search query from the action and params
            let query = action.split('_').next().unwrap_or(action).to_lowercase();

            // Retrieve relevant memories with a default limit of 10 (P4-002B)
            // Params are folded into the query so user-supplied context
            // (e.g. a "query" parameter) sharpens retrieval instead of being ignored.
            let query = if let Some(q) = params.get("query")
                && !q.is_empty()
            {
                format!("{} {}", query, q.to_lowercase())
            } else {
                query
            };
            let results = retrieval.retrieve(&query).await;

            if !results.is_empty() {
                tracing::info!(
                    "[Memory] Found {} relevant memories before action '{}'",
                    results.len(),
                    action
                );

                // Return memories as ToolOutput for the workflow executor to use
                return Some(ToolOutput::success(serde_json::json!({
                    "memories": results
                        .iter()
                        .map(|r| serde_json::json!({
                            "id": r.item.id,
                            "content": r.item.content,
                            "relevance_score": r.relevance_score,
                            "memory_type": r.item.memory_type.to_string(),
                        }))
                        .collect::<Vec<_>>(),
                })));
            }
        } else {
            tracing::trace!(
                "[Memory] No memory_retrieval available for action '{}', skipping memory read",
                action
            );
        }

        None
    }

    /// Record an experience after action completion
    pub async fn record_experience_after_action(
        &self,
        action: &str,
        params: &HashMap<String, String>,
        result: &ToolOutput,
    ) {
        if Self::should_skip_memory_read(action) {
            return;
        }

        let db = match &self.database {
            Some(db) => db,
            None => return,
        };

        // Use the shared coordinator if available, otherwise skip event recording
        let coordinator = match &self.coordinator {
            Some(c) => c,
            None => {
                tracing::trace!(
                    "[Experience] No coordinator available, skipping experience recording"
                );
                return;
            }
        };

        let outcome_kind = if result.success {
            OutcomeKind::Success
        } else {
            OutcomeKind::Failure
        };

        let title = build_experience_title(action, params);
        let description = build_experience_description(action, params, result);

        tracing::info!(
            "[Experience] Recording: {} - Outcome: {:?}",
            title,
            outcome_kind
        );

        let input = crate::bridge::tools::experience::RecordExperienceInput {
            title,
            description,
            experience_type: map_action_to_experience_type(action),
            outcome: outcome_kind,
            context: None,
            confidence: None,
            importance: None,
            tags: None,
        };

        // Use the shared coordinator - events will flow to WorkerManager and EventSubscriber
        match crate::bridge::tools::experience::execute_record_experience(input, coordinator, db)
            .await
        {
            Ok(_) => {
                tracing::debug!(
                    "[Experience] Recorded for future reflection/curation: action='{}'",
                    action
                );
            }
            Err(e) => {
                tracing::warn!("[Experience] Failed to record: {}", e);
            }
        }
    }
}

/// Build experience title from action and parameters
pub fn build_experience_title(action: &str, params: &HashMap<String, String>) -> String {
    let subject = params
        .get("title")
        .or(params.get("name"))
        .or(params.get("path"))
        .or(params.get("file_path"))
        .or(params.get("command"))
        .cloned()
        .unwrap_or_else(|| action.replace('_', " "));

    format!("Workflow: {}", subject)
}
