// src/workflows/engine/executor/experience.rs
//! Experience recording for workflow actions

use std::collections::HashMap;

use crate::experience::types::OutcomeKind;
use crate::tools::ToolOutput;
use crate::workflows::engine::SKIP_MEMORY_READ;
use crate::workflows::engine::types::WorkflowEngine;

use crate::workflows::engine::experience::{build_experience_description, map_action_to_experience_type};

impl WorkflowEngine {
    /// Check if action should skip memory read
    pub fn should_skip_memory_read(action: &str) -> bool {
        SKIP_MEMORY_READ.contains(&action)
    }

    /// Automatically read relevant memories before executing an action
    #[allow(unused)]
    pub async fn read_memory_before_action(
        &self,
        _action: &str,
        _params: &HashMap<String, String>,
    ) -> Option<ToolOutput> {
        // Workflow engine doesn't have memory_retrieval, so skip memory read
        // This is a limitation - workflow execution won't have context from working memory
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
                tracing::trace!("[Experience] No coordinator available, skipping experience recording");
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

        tracing::info!("[Experience] Recording: {} - Outcome: {:?}", title, outcome_kind);

        let input = crate::tools::experience::RecordExperienceInput {
            title,
            description,
            experience_type: map_action_to_experience_type(action),
            outcome: outcome_kind,
            context: None,
        };

        // Use the shared coordinator - events will flow to WorkerManager and EventSubscriber
        match crate::tools::experience::execute_record_experience(input, coordinator, db).await {
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
