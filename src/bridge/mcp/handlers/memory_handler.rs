// src/bridge/tools/handlers/memory_handler.rs
// Memory tools handler - handles memory operations and vector embeddings

use std::sync::Arc;
use crate::bridge::mcp::McpContext;
use crate::bridge::tools::memory;
use crate::bridge::mcp::handlers::{HandlerInitError, HandlerInitResult, ToolHandler};
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
}
