    // memory_tools.rs - Memory storage and retrieval tools

use crate::bridge::rmcp::types::McpServerHandler;
use crate::bridge::tools;
use crate::bridge::tools::ToolOutput;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_router;
use rmcp::tool;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};

#[tool_router]
impl McpServerHandler {
    #[tool(
        name = "get_workflow",
        description = "MANDATORY: Get workflow rules. MUST be called before any other tool."
    )]
    async fn get_workflow(
        &self,
        Parameters(input): Parameters<tools::agent::GetWorkflowInput>,
    ) -> ContentBlock {
        self.enforcer.record_workflow_retrieved(&self.session_id, input.purpose.clone()).await;
        match tools::agent::execute_get_workflow(input).await {
            Ok(result) => {
                self.record_tool_execution("get_workflow", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "store_memory", description = "Store a new memory in the knowledge base")]
    async fn store_memory(
        &self,
        Parameters(input): Parameters<tools::memory::StoreMemoryInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("store_memory").await {
            tracing::warn!("Workflow enforcement blocked store_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_store_memory(input, &self.context.database, &self.context.working_memory).await {
            Ok(result) => {
                self.record_tool_execution("store_memory", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "search_memory", description = "Search memories by content")]
    async fn search_memory(
        &self,
        Parameters(input): Parameters<tools::memory::SearchMemoryInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("search_memory").await {
            tracing::warn!("Workflow enforcement blocked search_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        let query = Some(input.query.clone());
        match tools::memory::execute_search_memory(input, &self.context.database, &self.context.memory_retrieval).await {
            Ok(result) => {
                self.record_tool_execution("search_memory", query).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_memory", description = "Get a specific memory by ID")]
    async fn get_memory(
        &self,
        Parameters(input): Parameters<tools::memory::GetMemoryInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_memory").await {
            tracing::warn!("Workflow enforcement blocked get_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_get_memory(input, &self.context.database, &self.context.memory_retrieval).await {
            Ok(result) => {
                self.record_tool_execution("get_memory", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "list_memories", description = "List recent memories")]
    async fn list_memories(
        &self,
        Parameters(input): Parameters<tools::memory::ListMemoriesInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("list_memories").await {
            tracing::warn!("Workflow enforcement blocked list_memories: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_list_memories(input, &self.context.database, &self.context.memory_retrieval).await {
            Ok(result) => {
                self.record_tool_execution("list_memories", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "store_embedding", description = "Store a vector embedding for semantic memory search")]
    async fn store_embedding(
        &self,
        Parameters(input): Parameters<tools::memory::StoreEmbeddingInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("store_embedding").await {
            tracing::warn!("Workflow enforcement blocked store_embedding: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_store_embedding(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("store_embedding", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_embedding", description = "Get an embedding by memory ID")]
    async fn get_embedding(
        &self,
        Parameters(input): Parameters<tools::memory::GetEmbeddingInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_embedding").await {
            tracing::warn!("Workflow enforcement blocked get_embedding: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_get_embedding(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_embedding", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "search_similar", description = "Search for similar memories using vector similarity")]
    async fn search_similar(
        &self,
        Parameters(input): Parameters<tools::memory::SearchSimilarInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("search_similar").await {
            tracing::warn!("Workflow enforcement blocked search_similar: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_search_similar(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("search_similar", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "list_embeddings", description = "List all memory embeddings")]
    async fn list_embeddings(
        &self,
        Parameters(input): Parameters<tools::memory::ListEmbeddingsInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("list_embeddings").await {
            tracing::warn!("Workflow enforcement blocked list_embeddings: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_list_embeddings(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("list_embeddings", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "delete_embedding", description = "Delete an embedding by memory ID")]
    async fn delete_embedding(
        &self,
        Parameters(input): Parameters<tools::memory::DeleteEmbeddingInput>,
    ) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("delete_embedding").await {
            tracing::warn!("Workflow enforcement blocked delete_embedding: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_delete_embedding(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("delete_embedding", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(name = "get_embedding_stats", description = "Get vector index statistics")]
    async fn get_embedding_stats(&self) -> ContentBlock {
        if let Err(e) = self.check_workflow_enforcement("get_embedding_stats").await {
            tracing::warn!("Workflow enforcement blocked get_embedding_stats: {}", e.message);
            return enforcement_error_to_content(e);
        }
        match tools::memory::execute_get_embedding_stats(&self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("get_embedding_stats", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }
}
