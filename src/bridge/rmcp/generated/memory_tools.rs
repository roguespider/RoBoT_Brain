// memory_tools.rs - Memory storage and retrieval tools
// This module loads independently and won't block MCP or other tools if it has issues.

use crate::bridge::rmcp::helpers::tool_output_to_content;
use crate::bridge::rmcp::generated::tool_traits::{
    MemoryToolsHandlerTrait, ToolContext,
};
use crate::tools;
use crate::tools::ToolOutput;

/// Handler for memory tools - implements MemoryToolsHandlerTrait
pub struct MemoryToolsHandler;

impl MemoryToolsHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryToolsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryToolsHandlerTrait for MemoryToolsHandler {
    async fn execute_get_workflow(
        &self,
        context: &ToolContext,
        input: tools::agent::GetWorkflowInput,
    ) -> ToolOutput {
        let mut enforcer = context.enforcer.lock().await;
        enforcer.record_workflow_retrieved(&context.session_id, input.purpose.clone());
        drop(enforcer);
        
        match tools::agent::execute_get_workflow(input).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_store_memory(
        &self,
        context: &ToolContext,
        input: tools::memory::StoreMemoryInput,
    ) -> ToolOutput {
        match tools::memory::execute_store_memory(
            input,
            &context.context.database,
            &context.context.working_memory,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_search_memory(
        &self,
        context: &ToolContext,
        input: tools::memory::SearchMemoryInput,
    ) -> ToolOutput {
        match tools::memory::execute_search_memory(
            input,
            &context.context.database,
            &context.context.memory_retrieval,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_memory(
        &self,
        context: &ToolContext,
        input: tools::memory::GetMemoryInput,
    ) -> ToolOutput {
        match tools::memory::execute_get_memory(
            input,
            &context.context.database,
            &context.context.memory_retrieval,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_memories(
        &self,
        context: &ToolContext,
        input: tools::memory::ListMemoriesInput,
    ) -> ToolOutput {
        match tools::memory::execute_list_memories(
            input,
            &context.context.database,
            &context.context.memory_retrieval,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_store_embedding(
        &self,
        context: &ToolContext,
        input: tools::memory::StoreEmbeddingInput,
    ) -> ToolOutput {
        match tools::memory::execute_store_embedding(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_embedding(
        &self,
        context: &ToolContext,
        input: tools::memory::GetEmbeddingInput,
    ) -> ToolOutput {
        match tools::memory::execute_get_embedding(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_search_similar(
        &self,
        context: &ToolContext,
        input: tools::memory::SearchSimilarInput,
    ) -> ToolOutput {
        match tools::memory::execute_search_similar(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_list_embeddings(
        &self,
        context: &ToolContext,
        input: tools::memory::ListEmbeddingsInput,
    ) -> ToolOutput {
        match tools::memory::execute_list_embeddings(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_delete_embedding(
        &self,
        context: &ToolContext,
        input: tools::memory::DeleteEmbeddingInput,
    ) -> ToolOutput {
        match tools::memory::execute_delete_embedding(input, &context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    async fn execute_get_embedding_stats(&self, context: &ToolContext) -> ToolOutput {
        match tools::memory::execute_get_embedding_stats(&context.context.database).await {
            Ok(result) => result,
            Err(e) => ToolOutput::error(e),
        }
    }

    fn list_tools(&self) -> Vec<rmcp::tool::Tool> {
        vec![
            tools::memory::get_workflow_tool(),
            tools::memory::store_memory_tool(),
            tools::memory::search_memory_tool(),
            tools::memory::get_memory_tool(),
            tools::memory::list_memories_tool(),
            tools::memory::store_embedding_tool(),
            tools::memory::get_embedding_tool(),
            tools::memory::search_similar_tool(),
            tools::memory::list_embeddings_tool(),
            tools::memory::delete_embedding_tool(),
            tools::memory::get_embedding_stats_tool(),
        ]
    }
}
