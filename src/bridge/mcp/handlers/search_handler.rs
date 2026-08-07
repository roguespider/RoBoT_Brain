// src/bridge/tools/handlers/search_handler.rs
// Search tools handler - handles global search and recommendations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::search;
use crate::bridge::mcp::handlers::{HandlerInitError, HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for search-related tools
#[derive(Clone)]
pub struct SearchToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl SearchToolsHandler {
    /// Create a new search tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "search",
                "Database connection not available",
            ));
        }

        Ok(Self { context, enforcer })
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
}
