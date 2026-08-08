// src/bridge/tools/handlers/reflection_handler.rs
// Reflection tools handler - handles reflection and pattern analysis tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::reflection;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for reflection-related tools
#[derive(Clone)]
pub struct ReflectionToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl ReflectionToolsHandler {
    /// Create a new reflection tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        Ok(Self { context, enforcer })
    }

    /// Get insights
    pub async fn execute_get_insights(
        &self,
        input: reflection::GetInsightsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_get_insights(input, &self.context.reflection).await
    }

    /// Create a reflection
    pub async fn execute_create_reflection(
        &self,
        input: reflection::CreateReflectionInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_create_reflection(input, &self.context.reflection).await
    }

    /// Analyze patterns
    pub async fn execute_analyze_patterns(
        &self,
        input: reflection::AnalyzePatternsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_analyze_patterns(input, &self.context.reflection).await
    }

    /// Get patterns
    pub async fn execute_get_patterns(
        &self,
        input: reflection::GetPatternsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_get_patterns(input, &self.context.reflection).await
    }
}

impl ToolHandler for ReflectionToolsHandler {
    fn category(&self) -> &str {
        "reflection"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "get_insights".to_string(),
            "create_reflection".to_string(),
            "analyze_patterns".to_string(),
            "get_patterns".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "get_insights" => {
                    let input: reflection::GetInsightsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_insights(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "create_reflection" => {
                    let input: reflection::CreateReflectionInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_create_reflection(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "analyze_patterns" => {
                    let input: reflection::AnalyzePatternsInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_analyze_patterns(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_patterns" => {
                    let input: reflection::GetPatternsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_patterns(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
