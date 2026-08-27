// src/workflows/engine/executor/execute.rs
//! Workflow step execution

use anyhow::Result;
use std::collections::HashMap;

use crate::workflows::engine::executor::variables::replace_variables;
use crate::workflows::engine::types::{WorkflowEngine, WorkflowStatus};

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

        let mut step_results: HashMap<String, crate::bridge::tools::ToolOutput> = HashMap::new();

        for step in &steps {
            tracing::info!(
                "Executing workflow {} step: {} (action: {})",
                workflow_id,
                step.name,
                step.action
            );

            // Replace variables in parameters
            let params = replace_variables(&step.parameters, &variables, &step_results);

            // Memory middleware: read before action (with error handling)
            let memory_context = self.read_memory_before_action(&step.action, &params).await;

            if let Some(ref ctx) = memory_context
                && let Some(memories) = ctx.data.get("memories").and_then(|v| v.as_array())
                && !memories.is_empty()
            {
                tracing::info!(
                    "Found {} relevant memories before action '{}'",
                    memories.len(),
                    step.action
                );
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
}
