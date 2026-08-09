// src/bridge/tools/handlers/knowledge_handler.rs
// Knowledge tools handler - handles knowledge base operations

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::knowledge;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitResult, ToolHandler};

/// Handler for knowledge-related tools
#[derive(Clone)]
pub struct KnowledgeToolsHandler {
    context: Arc<McpContext>,
}

impl KnowledgeToolsHandler {
    /// Create a new knowledge tools handler
    pub fn new(
        context: Arc<McpContext>,
    ) -> HandlerInitResult<Self> {
        // Knowledge store is available - async validation happens at runtime
        Ok(Self { context })
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
        true
    }

    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        use crate::bridge::mcp::handlers::json_to_schema;
        vec![
            rmcp::model::Tool::new(
                "add_knowledge",
                "Add new validated knowledge to the knowledge base",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "statement": { "type": "string", "description": "The knowledge statement" },
                        "knowledge_type": { "type": "string", "description": "Type: fact, procedure, causality, pattern, insight, rule, concept" },
                        "source": { "type": "string", "description": "Source of the knowledge" },
                        "confidence": { "type": "number", "description": "Initial confidence (0.0-1.0)" },
                        "tags": { "type": "array", "items": { "type": "string" }, "description": "Tags" }
                    },
                    "required": ["statement"]
                })),
            ).with_title("Add Knowledge"),
            rmcp::model::Tool::new(
                "query_knowledge",
                "Query the knowledge base for relevant knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "knowledge_type": { "type": "string", "description": "Filter by type" },
                        "limit": { "type": "number", "description": "Maximum results" },
                        "min_confidence": { "type": "number", "description": "Minimum confidence" }
                    },
                    "required": ["query"]
                })),
            ).with_title("Query Knowledge"),
            rmcp::model::Tool::new(
                "record_knowledge_application",
                "Record the result of applying knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "knowledge_id": { "type": "string", "description": "Knowledge ID" },
                        "success": { "type": "boolean", "description": "Whether application was successful" }
                    },
                    "required": ["knowledge_id", "success"]
                })),
            ).with_title("Record Knowledge Application"),
            rmcp::model::Tool::new(
                "get_knowledge_stats",
                "Get knowledge base statistics",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {}
                })),
            ).with_title("Get Knowledge Stats"),
            rmcp::model::Tool::new(
                "get_mature_knowledge",
                "Get mature (high-confidence) knowledge",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "number", "description": "Maximum results" }
                    }
                })),
            ).with_title("Get Mature Knowledge"),
        ]
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
