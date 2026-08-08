// src/bridge/tools/handlers/knowledge_handler.rs
// Knowledge tools handler - handles knowledge base operations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::knowledge;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for knowledge-related tools
#[derive(Clone)]
pub struct KnowledgeToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl KnowledgeToolsHandler {
    /// Create a new knowledge tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Knowledge store is available - async validation happens at runtime
        Ok(Self { context, enforcer })
    }

    /// Add new validated knowledge
    pub async fn execute_add_knowledge(
        &self,
        input: knowledge::AddKnowledgeInput,
    ) -> crate::bridge::tools::ToolOutput {
        knowledge::execute_add_knowledge(input, &self.context.knowledge).await
    }

    /// Query the knowledge base
    pub async fn execute_query_knowledge(
        &self,
        input: knowledge::QueryKnowledgeInput,
    ) -> crate::bridge::tools::ToolOutput {
        knowledge::execute_query_knowledge(input, &self.context.knowledge).await
    }

    /// Record knowledge application result
    pub async fn execute_record_knowledge_application(
        &self,
        input: knowledge::RecordKnowledgeApplicationInput,
    ) -> crate::bridge::tools::ToolOutput {
        knowledge::execute_record_knowledge_application(input, &self.context.knowledge).await
    }

    /// Get knowledge statistics
    pub async fn execute_get_knowledge_stats(
        &self,
        input: knowledge::GetKnowledgeStatsInput,
    ) -> crate::bridge::tools::ToolOutput {
        knowledge::execute_get_knowledge_stats(input, &self.context.knowledge).await
    }

    /// Get mature (high-confidence) knowledge
    pub async fn execute_get_mature_knowledge(
        &self,
        input: knowledge::GetMatureKnowledgeInput,
    ) -> crate::bridge::tools::ToolOutput {
        knowledge::execute_get_mature_knowledge(input, &self.context.knowledge).await
    }
}

impl ToolHandler for KnowledgeToolsHandler {
    fn category(&self) -> &str {
        "knowledge"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "add_knowledge".to_string(),
            "query_knowledge".to_string(),
            "record_knowledge_application".to_string(),
            "get_knowledge_stats".to_string(),
            "get_mature_knowledge".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        // Health check would be async - for now return true if context is available
        true
    }

    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "add_knowledge" => {
                    let input: knowledge::AddKnowledgeInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_add_knowledge(input).await)
                }
                "query_knowledge" => {
                    let input: knowledge::QueryKnowledgeInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_query_knowledge(input).await)
                }
                "record_knowledge_application" => {
                    let input: knowledge::RecordKnowledgeApplicationInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    Ok(self.execute_record_knowledge_application(input).await)
                }
                "get_knowledge_stats" => {
                    let input: knowledge::GetKnowledgeStatsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    Ok(self.execute_get_knowledge_stats(input).await)
                }
                "get_mature_knowledge" => {
                    let input: knowledge::GetMatureKnowledgeInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    Ok(self.execute_get_mature_knowledge(input).await)
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
