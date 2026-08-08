// src/bridge/tools/handlers/memory_handler.rs
// Memory tools handler - handles memory operations and vector embeddings

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::memory;
use crate::bridge::mcp::handlers::{HandlerError, HandlerInitError, HandlerInitResult, ToolHandler, json_to_schema};
use crate::workflows::enforcement::WorkflowEnforcer;

/// Handler for memory-related tools
#[derive(Clone)]
pub struct MemoryToolsHandler {
    context: Arc<McpContext>,
    enforcer: Arc<WorkflowEnforcer>,
}

impl MemoryToolsHandler {
    /// Create a new memory tools handler
    pub fn new(
        context: Arc<McpContext>,
        enforcer: Arc<WorkflowEnforcer>,
    ) -> HandlerInitResult<Self> {
        // Validate that required dependencies exist
        if context.database.connection().is_err() {
            return Err(HandlerInitError::new(
                "memory",
                "Database connection not available",
            ));
        }

        Ok(Self { context, enforcer })
    }

    /// Store a new memory
    pub async fn execute_store_memory(
        &self,
        input: memory::StoreMemoryInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_store_memory(
            input,
            &self.context.database,
            &self.context.working_memory,
        )
        .await
    }

    /// Search memories by content
    pub async fn execute_search_memory(
        &self,
        input: memory::SearchMemoryInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_search_memory(
            input,
            &self.context.database,
            &self.context.memory_retrieval,
        )
        .await
    }

    /// Get a specific memory by ID
    pub async fn execute_get_memory(
        &self,
        input: memory::GetMemoryInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_get_memory(
            input,
            &self.context.database,
            &self.context.memory_retrieval,
        )
        .await
    }

    /// List recent memories
    pub async fn execute_list_memories(
        &self,
        input: memory::ListMemoriesInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_list_memories(
            input,
            &self.context.database,
            &self.context.memory_retrieval,
        )
        .await
    }

    /// Store a vector embedding
    pub async fn execute_store_embedding(
        &self,
        input: memory::StoreEmbeddingInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_store_embedding(input, &self.context.database).await
    }

    /// Get an embedding by memory ID
    pub async fn execute_get_embedding(
        &self,
        input: memory::GetEmbeddingInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_get_embedding(input, &self.context.database).await
    }

    /// Search for similar memories using vector similarity
    pub async fn execute_search_similar(
        &self,
        input: memory::SearchSimilarInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_search_similar(input, &self.context.database).await
    }

    /// List all memory embeddings
    pub async fn execute_list_embeddings(
        &self,
        input: memory::ListEmbeddingsInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_list_embeddings(input, &self.context.database).await
    }

    /// Delete an embedding by memory ID
    pub async fn execute_delete_embedding(
        &self,
        input: memory::DeleteEmbeddingInput,
    ) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_delete_embedding(input, &self.context.database).await
    }

    /// Get embedding statistics
    pub async fn execute_get_embedding_stats(&self) -> Result<crate::bridge::tools::ToolOutput, anyhow::Error> {
        memory::execute_get_embedding_stats(&self.context.database).await
    }
}

impl ToolHandler for MemoryToolsHandler {
    fn category(&self) -> &str {
        "memory"
    }

    fn tool_names(&self) -> Vec<String> {
        vec![
            "store_memory".to_string(),
            "search_memory".to_string(),
            "get_memory".to_string(),
            "list_memories".to_string(),
            "store_embedding".to_string(),
            "get_embedding".to_string(),
            "search_similar".to_string(),
            "list_embeddings".to_string(),
            "delete_embedding".to_string(),
            "get_embedding_stats".to_string(),
        ]
    }

    fn is_healthy(&self) -> bool {
        self.context.database.connection().is_ok()
    }
    
    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        vec![
            rmcp::model::Tool::new(
                "store_memory",
                "Store a new memory in the agent's memory system",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "The memory content to store" },
                        "importance": { "type": "number", "description": "Importance score (0-10)" },
                        "category": { "type": "string", "description": "Category for the memory" }
                    },
                    "required": ["content"]
                })),
            ).with_title("Store Memory"),
            rmcp::model::Tool::new(
                "search_memory",
                "Search memories by content query",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query" },
                        "limit": { "type": "number", "description": "Maximum results to return" }
                    },
                    "required": ["query"]
                })),
            ).with_title("Search Memory"),
            rmcp::model::Tool::new(
                "get_memory",
                "Retrieve a specific memory by ID",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "Memory ID" }
                    },
                    "required": ["id"]
                })),
            ).with_title("Get Memory"),
            rmcp::model::Tool::new(
                "list_memories",
                "List all memories with optional filtering",
                json_to_schema(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "limit": { "type": "number", "description": "Maximum results" }
                    }
                })),
            ).with_title("List Memories"),
        ]
    }
    
    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            match name {
                "store_memory" => {
                    let input: memory::StoreMemoryInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_store_memory(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "search_memory" => {
                    let input: memory::SearchMemoryInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_search_memory(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_memory" => {
                    let input: memory::GetMemoryInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_memory(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_memories" => {
                    let input: memory::ListMemoriesInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_memories(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "store_embedding" => {
                    let input: memory::StoreEmbeddingInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_store_embedding(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_embedding" => {
                    let input: memory::GetEmbeddingInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_get_embedding(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "search_similar" => {
                    let input: memory::SearchSimilarInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_search_similar(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "list_embeddings" => {
                    let input: memory::ListEmbeddingsInput = serde_json::from_value(args)
                        .unwrap_or_default();
                    self.execute_list_embeddings(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "delete_embedding" => {
                    let input: memory::DeleteEmbeddingInput = serde_json::from_value(args)
                        .map_err(|e| HandlerError::InvalidParams(e.to_string()))?;
                    self.execute_delete_embedding(input).await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                "get_embedding_stats" => {
                    self.execute_get_embedding_stats().await
                        .map_err(|e| HandlerError::ExecutionFailed(e.to_string()))
                }
                _ => Err(HandlerError::ToolNotFound(name.to_string()))
            }
        }
    }
}
