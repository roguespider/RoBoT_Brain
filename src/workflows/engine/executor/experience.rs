// src/workflows/engine/executor/experience.rs
//! Experience recording for workflow actions

use std::collections::HashMap;

use crate::experience::types::OutcomeKind;
use crate::bridge::tools::ToolOutput;
use crate::workflows::engine::types::WorkflowEngine;
use crate::workflows::engine::SKIP_MEMORY_READ;

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
        // Check if coordinator is available for memory retrieval
        if self.coordinator.is_none() {
            tracing::trace!(
                "[Memory] No coordinator available for action '{}', skipping memory read",
                action
            );
            return None;
        }

        // Extract any memory ID from params that might indicate what to read
        let memory_hint = params.get("memory_id").or_else(|| params.get("memory_context"));
        
        if let Some(hint) = memory_hint {
            tracing::debug!(
                "[Memory] Would read memory hint '{}' before action '{}'",
                hint,
                action
            );
            // Memory retrieval integration pending - coordinator not fully connected to memory system
        }

        // Log the action context for debugging
        tracing::trace!(
            "[Memory] Checking memory context for action '{}' with {} parameters",
            action,
            params.len()
        );

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
        match crate::bridge::tools::experience::execute_record_experience(input, coordinator, db).await {
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
