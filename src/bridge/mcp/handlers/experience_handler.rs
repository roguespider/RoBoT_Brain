// src/bridge/tools/handlers/experience_handler.rs
// Experience tools handler - handles experience recording and worker management

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::experience;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitError, HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for experience-related tools
#[derive(Clone)]
pub struct ExperienceToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl ExperienceToolsHandler {
    /// Create a new experience tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "experience",
                "Database connection not available",
            ));
        }

        Ok(Self { context, enforcer })
    }

    /// Record a new experience
    pub async fn execute_record_experience(
        &self,
        input: experience::RecordExperienceInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        experience::execute_record_experience(
            input,
            &self.context.coordinator,
            &self.context.database,
        )
        .await
    }

    /// Get experience statistics
    pub async fn execute_get_experience_stats(
        &self,
        input: experience::GetExperienceStatsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        experience::execute_get_experience_stats(input, &self.context.database).await
    }

    /// List recent experiences
    pub async fn execute_list_experiences(
        &self,
        input: experience::ListExperiencesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        experience::execute_list_experiences(input, &self.context.database).await
    }

    /// Get a specific experience by ID
    pub async fn execute_get_experience(
        &self,
        input: experience::GetExperienceInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        experience::execute_get_experience(input, &self.context.database).await
    }

    /// Get background worker statistics
    pub async fn execute_get_worker_stats(
        &self,
        input: experience::GetWorkerStatsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        experience::execute_get_worker_stats(input, &self.context.worker_manager).await
    }

    /// Get the number of active background workers
    pub async fn execute_get_worker_count(&self) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        experience::execute_get_worker_count(&self.context.worker_manager).await
    }
}

impl ToolHandler for ExperienceToolsHandler {
    fn category(&self) -> &str {
        "experience"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "record_experience".to_string(),
            "get_experience_stats".to_string(),
            "list_experiences".to_string(),
            "get_experience".to_string(),
            "get_worker_stats".to_string(),
            "get_worker_count".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "record_experience" => {
                    let input: experience::RecordExperienceInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_record_experience(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_experience_stats" => {
                    let input: experience::GetExperienceStatsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_experience_stats(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_experiences" => {
                    let input: experience::ListExperiencesInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_experiences(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_experience" => {
                    let input: experience::GetExperienceInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_experience(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_worker_stats" => {
                    let input: experience::GetWorkerStatsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_worker_stats(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_worker_count" => {
                    self.execute_get_worker_count().await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
