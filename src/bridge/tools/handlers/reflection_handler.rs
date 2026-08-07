// src/bridge/tools/handlers/reflection_handler.rs
// Reflection tools handler - handles reflection and pattern analysis tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::reflection;
use crate::bridge::tools::handlers::{HandlerInitResult, ToolHandler};
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
}
