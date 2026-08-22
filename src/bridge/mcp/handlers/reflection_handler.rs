// src/bridge/tools/handlers/reflection_handler.rs
// Reflection tools handler - handles reflection and pattern analysis tools

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::reflection;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};

/// Handler for reflection-related tools
#[derive(Clone)]
pub struct ReflectionToolsHandler {
    context: Arc<McpContext>,
}

impl ReflectionToolsHandler {
    /// Create a new reflection tools handler
    pub fn new(
        context: Arc<McpContext>,
    ) -> HandlerInitResult<Self> {
        Ok(Self { context })
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

    /// Validate a reflection
    pub async fn execute_validate_reflection(
        &self,
        input: reflection::ValidateReflectionInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_validate_reflection(input, &self.context.reflection).await
    }

    /// List reflections by status
    pub async fn execute_list_reflections_by_status(
        &self,
        input: reflection::ListReflectionsByStatusInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_list_reflections_by_status(input, &self.context.reflection).await
    }

    /// Update a reflection
    pub async fn execute_update_reflection(
        &self,
        input: reflection::UpdateReflectionInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        reflection::execute_update_reflection(input, &self.context.reflection).await
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
            "validate_reflection".to_string(),
            "list_reflections_by_status".to_string(),
            "update_reflection".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "get_insights",
                "Get actionable insights from past experiences",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "domain": { "type": "string", "description": "Filter by domain" },
                        "limit": { "type": "number", "description": "Maximum insights to return" }
                    }
                })),
            ).with_title("Get Insights"),
            rmcp::model::Tool::new(
                "create_reflection",
                "Create a reflection on recent experiences",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": { "type": "string", "description": "Reflection title" },
                        "content": { "type": "string", "description": "Reflection content" },
                        "experience_ids": { "type": "array", "items": { "type": "string" }, "description": "Related experience IDs" }
                    },
                    "required": ["title", "content"]
                })),
            ).with_title("Create Reflection"),
            rmcp::model::Tool::new(
                "analyze_patterns",
                "Analyze recent experiences for patterns",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "lookback_days": { "type": "number", "description": "Days to look back" }
                    }
                })),
            ).with_title("Analyze Patterns"),
            rmcp::model::Tool::new(
                "get_patterns",
                "Get detected patterns from analysis",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern_type": { "type": "string", "description": "Filter by pattern type" }
                    }
                })),
            ).with_title("Get Patterns"),
            rmcp::model::Tool::new(
                "validate_reflection",
                "Validate a reflection for quality and consistency",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "reflection_id": { "type": "string", "description": "ID of the reflection to validate" }
                    },
                    "required": ["reflection_id"]
                })),
            ).with_title("Validate Reflection"),
            rmcp::model::Tool::new(
                "list_reflections_by_status",
                "List reflections filtered by status",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "description": "Status: draft, validated, contradicted, or archived", "enum": ["draft", "active", "validated", "archived"] }
                    },
                    "required": ["status"]
                })),
            ).with_title("List Reflections By Status"),
            rmcp::model::Tool::new(
                "update_reflection",
                "Update an existing reflection's title, description, or summary",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "reflection_id": { "type": "string", "description": "ID of the reflection to update" },
                        "title": { "type": "string", "description": "New title" },
                        "description": { "type": "string", "description": "New description" },
                        "summary": { "type": "string", "description": "New summary" }
                    },
                    "required": ["reflection_id"]
                })),
            ).with_title("Update Reflection"),
        ]
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
                "validate_reflection" => {
                    let input: reflection::ValidateReflectionInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_validate_reflection(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_reflections_by_status" => {
                    let input: reflection::ListReflectionsByStatusInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_list_reflections_by_status(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "update_reflection" => {
                    let input: reflection::UpdateReflectionInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_update_reflection(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
