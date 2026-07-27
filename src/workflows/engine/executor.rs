// src/workflows/engine/executor.rs
//! Workflow execution methods

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde_json::json;

use crate::experience::types::OutcomeKind;
use crate::tools::{self, ToolOutput};
use crate::workflows::engine::types::{WorkflowEngine, WorkflowStatus};

use super::experience::{build_experience_description, map_action_to_experience_type};
use crate::workflows::engine::SKIP_MEMORY_READ;

impl WorkflowEngine {
    /// Execute workflow steps
    pub async fn execute_workflow(&self, workflow_id: &str) -> Result<()> {
        let (steps, mut variables) = {
            let workflows = self.workflows.read().await;
            let steps = workflows.get(workflow_id).map(|w| w.steps.clone());
            let vars = workflows
                .get(workflow_id)
                .map(|w| w.variables.clone())
                .unwrap_or_default();
            (steps, vars)
        };

        let Some(steps) = steps else {
            return Ok(());
        };

        let mut step_results: HashMap<String, ToolOutput> = HashMap::new();

        for step in &steps {
            tracing::info!(
                "Executing workflow {} step: {} (action: {})",
                workflow_id,
                step.name,
                step.action
            );

            // Replace variables in parameters
            let params = Self::replace_variables(&step.parameters, &variables, &step_results);

            // Memory middleware: read before action
            let memory_context = self.read_memory_before_action(&step.action, &params).await;

            if let Some(ref ctx) = memory_context {
                if let Some(memories) = ctx.data.get("memories").and_then(|v| v.as_array()) {
                    if !memories.is_empty() {
                        tracing::info!(
                            "Found {} relevant memories before action '{}'",
                            memories.len(),
                            step.action
                        );
                    }
                }
            }

            // Execute the step action
            let result = self.execute_step_action(&step.action, &params).await;

            match result {
                Ok(output) => {
                    tracing::info!("Step {} completed successfully", step.name);
                    step_results.insert(step.id.clone(), output.clone());
                    self.metrics.increment("workflows.steps.executed").await;

                    // Record experience for reflection
                    self.record_experience_after_action(&step.action, &params, &output)
                        .await;

                    if let Some(var_name) = &step.on_success {
                        variables.insert(
                            var_name.clone(),
                            serde_json::to_string(&output.data).unwrap_or_default(),
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Step {} failed: {}", step.name, e);
                    self.metrics.increment("workflows.steps.failed").await;

                    {
                        let mut workflows = self.workflows.write().await;
                        if let Some(workflow) = workflows.get_mut(workflow_id) {
                            workflow.status = WorkflowStatus::Failed;
                        }
                    }

                    {
                        let mut executing = self.executing.write().await;
                        executing.remove(workflow_id);
                    }
                    return Err(e);
                }
            }
        }

        // Mark workflow as completed
        {
            let mut workflows = self.workflows.write().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Completed;
                workflow.completed_at = Some(chrono::Utc::now());
                workflow.variables = variables;
            }
        }

        {
            let mut executing = self.executing.write().await;
            executing.remove(workflow_id);
        }

        self.metrics.increment("workflows.completed").await;

        Ok(())
    }

    /// Replace variables in parameters with their values
    pub fn replace_variables(
        params: &HashMap<String, String>,
        workflow_vars: &HashMap<String, String>,
        step_results: &HashMap<String, ToolOutput>,
    ) -> HashMap<String, String> {
        let mut resolved = params.clone();
        for value in resolved.values_mut() {
            // Replace workflow variables ${var_name}
            for (var_name, var_value) in workflow_vars {
                let placeholder = format!("${{{}}}", var_name);
                *value = value.replace(&placeholder, var_value);
            }
            // Replace step result references ${step_id.output_field}
            for (step_id, result) in step_results {
                if let Some(obj) = result.data.as_object() {
                    for (field, field_value) in obj {
                        let placeholder = format!("${{{}.{}}}", step_id, field);
                        *value = value.replace(&placeholder, &field_value.to_string());
                    }
                }
            }
        }
        resolved
    }

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

    /// Check if action should skip memory read
    pub fn should_skip_memory_read(action: &str) -> bool {
        SKIP_MEMORY_READ.iter().any(|&s| s == action)
    }

    /// Automatically read relevant memories before executing an action
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

        let outcome_kind = if result.success {
            OutcomeKind::Success
        } else {
            OutcomeKind::Failure
        };

        let title = build_experience_title(action, params);
        let description = build_experience_description(action, params, result);

        tracing::info!("[Experience] Recording: {} - Outcome: {:?}", title, outcome_kind);

        let input = tools::experience::RecordExperienceInput {
            title,
            description,
            experience_type: map_action_to_experience_type(action),
            outcome: outcome_kind,
            context: None,
        };

        let scorer = crate::experience::scorer::ExperienceScorer::new();
        let bus = Arc::new(crate::experience::bus::ExperienceBus::new());
        let metrics = Arc::new(crate::experience::metrics::MetricsCollector::new());
        let coordinator = Arc::new(crate::experience::coordinator::ExperienceCoordinator::new(
            scorer, bus, metrics,
        ));

        match tools::experience::execute_record_experience(input, &coordinator, db).await {
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
