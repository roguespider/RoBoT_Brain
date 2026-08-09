// src/bridge/tools/handlers/search_handler.rs
// Search tools handler - handles global search and recommendations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::search;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitError, HandlerInitResult, ToolHandler};

/// Handler for search-related tools
#[derive(Clone)]
pub struct SearchToolsHandler {
    context: Arc<McpContext>,
}

impl SearchToolsHandler {
    /// Create a new search tools handler
    pub fn new(
        context: Arc<McpContext>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "search",
                "Database connection not available",
            ));
        }

        Ok(Self { context })
    }

    /// Execute global search across all data
    pub async fn execute_global_search(
        &self,
        input: search::GlobalSearchInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        search::execute_global_search(input, &self.context.database).await
    }

    /// Get recommendations based on patterns
    pub async fn execute_get_recommendations(
        &self,
        input: search::GetRecommendationsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        search::execute_get_recommendations(input, &self.context.database).await
    }

    /// Get reputation score for a tool
    pub async fn execute_get_reputation(
        &self,
        input: search::GetReputationInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        search::execute_get_reputation(input, &self.context.database).await
    }
}

impl ToolHandler for SearchToolsHandler {
    fn category(&self) -> &str {
        "search"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "global_search".to_string(),
            "get_recommendations".to_string(),
            "get_reputation".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "global_search",
                "Search across all memories, experiences, and knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "limit": { "type": "number", "description": "Maximum results per category" }
                    },
                    "required": ["query"]
                })),
            ).with_title("Global Search"),
            rmcp::model::Tool::new(
                "get_recommendations",
                "Get recommendations based on patterns and history",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "category": { "type": "string", "description": "Recommendation category" }
                    }
                })),
            ).with_title("Get Recommendations"),
            rmcp::model::Tool::new(
                "get_reputation",
                "Get reputation/quality score for a tool or approach",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tool_name": { "type": "string", "description": "Tool name to check" }
                    },
                    "required": ["tool_name"]
                })),
            ).with_title("Get Reputation"),
        ]
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "global_search" => {
                    let input: search::GlobalSearchInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_global_search(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_recommendations" => {
                    let input: search::GetRecommendationsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_get_recommendations(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_reputation" => {
                    let input: search::GetReputationInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_reputation(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
