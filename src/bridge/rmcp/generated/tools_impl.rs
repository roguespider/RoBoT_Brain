// tools_impl.rs
// Tool implementations with #[tool_router] and #[tool] macros
// This file is included after server_handler_impl.rs

use std::sync::Arc;
use crate::bridge::rmcp::types::McpServerHandler;
use crate::bridge::rmcp::helpers::{tool_output_to_content, enforcement_error_to_content};
use crate::tools::{self, ToolOutput};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ContentBlock;
use rmcp::tool_handler;
use rmcp::tool_router;
use rmcp::tool;

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


#[tool(name = "record_experience", description = "Record a new experience")]
async fn record_experience(
    &self,
    Parameters(input): Parameters<tools::experience::RecordExperienceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("record_experience").await {
        tracing::warn!("Workflow enforcement blocked record_experience: {}", e.message);
        return enforcement_error_to_content(e);
    }
    
    match tools::experience::execute_record_experience(
        input,
        &self.context.coordinator,
        &self.context.database,
    ).await {
        Ok(result) => {
            self.record_tool_execution("record_experience", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_experience_stats", description = "Get experience statistics")]
async fn get_experience_stats(
    &self,
    Parameters(input): Parameters<tools::experience::GetExperienceStatsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_experience_stats").await {
        tracing::warn!("Workflow enforcement blocked get_experience_stats: {}", e.message);
        return enforcement_error_to_content(e);
    }
    
    match tools::experience::execute_get_experience_stats(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_experience_stats", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_experiences", description = "List recent experiences")]
async fn list_experiences(
    &self,
    Parameters(input): Parameters<tools::experience::ListExperiencesInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_experiences").await {
        tracing::warn!("Workflow enforcement blocked list_experiences: {}", e.message);
        return enforcement_error_to_content(e);
    }
    
    match tools::experience::execute_list_experiences(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_experiences", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_experience", description = "Get a specific experience by ID")]
async fn get_experience(
    &self,
    Parameters(input): Parameters<tools::experience::GetExperienceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_experience").await {
        tracing::warn!("Workflow enforcement blocked get_experience: {}", e.message);
        return enforcement_error_to_content(e);
    }
    
    match tools::experience::execute_get_experience(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_experience", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_worker_stats", description = "Get background worker statistics")]
async fn get_worker_stats(
    &self,
    Parameters(input): Parameters<tools::experience::GetWorkerStatsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_worker_stats").await {
        tracing::warn!("Workflow enforcement blocked get_worker_stats: {}", e.message);
        return enforcement_error_to_content(e);
    }

    match tools::experience::execute_get_worker_stats(input, &self.context.worker_manager).await {
        Ok(result) => {
            self.record_tool_execution("get_worker_stats", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_worker_count", description = "Get the number of active background workers")]
async fn get_worker_count(&self) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_worker_count").await {
        tracing::warn!("Workflow enforcement blocked get_worker_count: {}", e.message);
        return enforcement_error_to_content(e);
    }

    match tools::experience::execute_get_worker_count(&self.context.worker_manager).await {
        Ok(result) => {
            self.record_tool_execution("get_worker_count", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}


#[tool(name = "get_insights", description = "Get actionable insights from reflections")]
async fn get_insights(
    &self,
    Parameters(input): Parameters<tools::reflection::GetInsightsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_insights").await {
        tracing::warn!("Workflow enforcement blocked get_insights: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_get_insights(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("get_insights", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "create_reflection", description = "Create a new reflection")]
async fn create_reflection(
    &self,
    Parameters(input): Parameters<tools::reflection::CreateReflectionInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("create_reflection").await {
        tracing::warn!("Workflow enforcement blocked create_reflection: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_create_reflection(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("create_reflection", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "analyze_patterns", description = "Analyze experiences to detect patterns")]
async fn analyze_patterns(
    &self,
    Parameters(input): Parameters<tools::reflection::AnalyzePatternsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("analyze_patterns").await {
        tracing::warn!("Workflow enforcement blocked analyze_patterns: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_analyze_patterns(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("analyze_patterns", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_patterns", description = "Get detected patterns")]
async fn get_patterns(
    &self,
    Parameters(input): Parameters<tools::reflection::GetPatternsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_patterns").await {
        tracing::warn!("Workflow enforcement blocked get_patterns: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::reflection::execute_get_patterns(input, &self.context.reflection).await {
        Ok(result) => {
            self.record_tool_execution("get_patterns", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}


#[tool(name = "global_search", description = "Search across all memories and experiences")]
async fn global_search(
    &self,
    Parameters(input): Parameters<tools::search::GlobalSearchInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("global_search").await {
        tracing::warn!("Workflow enforcement blocked global_search: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let query = Some(input.query.clone());
    match tools::search::execute_global_search(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("global_search", query).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_recommendations", description = "Get recommendations based on learned patterns")]
async fn get_recommendations(
    &self,
    Parameters(input): Parameters<tools::search::GetRecommendationsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_recommendations").await {
        tracing::warn!("Workflow enforcement blocked get_recommendations: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::search::execute_get_recommendations(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_recommendations", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_reputation", description = "Get reputation score for a target")]
async fn get_reputation(
    &self,
    Parameters(input): Parameters<tools::search::GetReputationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_reputation").await {
        tracing::warn!("Workflow enforcement blocked get_reputation: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::search::execute_get_reputation(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_reputation", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}


#[tool(
    name = "ingest_files",
    description = "Ingest files from files_to_import folder into memory."
)]
async fn ingest_files(
    &self,
    Parameters(input): Parameters<tools::ingestor::IngestFilesInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("ingest_files").await {
        tracing::warn!("Workflow enforcement blocked ingest_files: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::ingestor::ingest_file(
        input, 
        Arc::clone(&self.context.database),
        Arc::clone(&self.context.working_memory),
    ).await {
        Ok(result) => {
            self.record_tool_execution("ingest_files", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_importable", description = "List files available for import.")]
async fn list_importable(
    &self,
    Parameters(input): Parameters<tools::ingestor::ListImportableInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_importable").await {
        tracing::warn!("Workflow enforcement blocked list_importable: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::ingestor::execute_list_importable(input).await {
        Ok(result) => {
            self.record_tool_execution("list_importable", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "transcribe_audio", description = "Transcribe an audio file to text using Whisper AI. Requires audio feature.")]
async fn transcribe_audio(
    &self,
    Parameters(input): Parameters<tools::ingestor::TranscribeAudioInput>,
) -> ContentBlock {
    let error_msg = format!(
        "Audio transcription requires the 'audio' feature to be enabled. Cannot transcribe: {}",
        input.path
    );
    tool_output_to_content(ToolOutput::error(error_msg))
}

#[tool(name = "list_ingested_files", description = "List files that have been successfully ingested.")]
async fn list_ingested_files(
    &self,
    Parameters(input): Parameters<tools::ingestor::ListIngestedFilesInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_ingested_files").await {
        tracing::warn!("Workflow enforcement blocked list_ingested_files: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::ingestor::execute_list_ingested_files(input).await {
        Ok(result) => {
            self.record_tool_execution("list_ingested_files", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "delete_ingested_files",
    description = "Delete original files after successful ingestion. Requires confirmation='yes'."
)]
async fn delete_ingested_files(
    &self,
    Parameters(input): Parameters<tools::ingestor::DeleteIngestedFilesInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("delete_ingested_files").await {
        tracing::warn!("Workflow enforcement blocked delete_ingested_files: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::ingestor::execute_delete_ingested_files(input).await {
        Ok(result) => {
            self.record_tool_execution("delete_ingested_files", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}


#[tool(
    name = "list_tools",
    description = "List all available MCP tools with optional filter"
)]
async fn list_tools(
    &self,
    Parameters(input): Parameters<tools::agent::ListToolsInput>,
) -> ContentBlock {
    match tools::agent::execute_list_tools(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "get_tool",
    description = "Get detailed information about a specific tool"
)]
async fn get_tool(
    &self,
    Parameters(input): Parameters<tools::agent::GetToolInput>,
) -> ContentBlock {
    match tools::agent::execute_get_tool(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "connect_mcp_server",
    description = "Connect to an external MCP server via child process"
)]
async fn connect_mcp_server(
    &self,
    Parameters(input): Parameters<tools::agent::ConnectMcpServerInput>,
) -> ContentBlock {
    match tools::agent::execute_connect_mcp_server(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(
    name = "call_tool",
    description = "Call a tool on a connected MCP server"
)]
async fn call_tool(
    &self,
    Parameters(input): Parameters<tools::agent::CallMcpToolInput>,
) -> ContentBlock {
    match tools::agent::execute_call_mcp_tool(input).await {
        Ok(result) => tool_output_to_content(result),
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}


#[tool(
    name = "record_observation",
    description = "Record an observation. Observations are the starting point for learning."
)]
async fn record_observation(
    &self,
    Parameters(input): Parameters<tools::hypothesis::RecordObservationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("record_observation").await {
        tracing::warn!("Workflow enforcement blocked record_observation: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_record_observation(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("record_observation", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "create_hypothesis", description = "Create a testable hypothesis from observations.")]
async fn create_hypothesis(
    &self,
    Parameters(input): Parameters<tools::hypothesis::CreateHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("create_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked create_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_create_hypothesis(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("create_hypothesis", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "add_evidence", description = "Add evidence to a hypothesis.")]
async fn add_evidence(
    &self,
    Parameters(input): Parameters<tools::hypothesis::AddEvidenceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_evidence").await {
        tracing::warn!("Workflow enforcement blocked add_evidence: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_add_evidence(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("add_evidence", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_hypothesis", description = "Get details of a specific hypothesis.")]
async fn get_hypothesis(
    &self,
    Parameters(input): Parameters<tools::hypothesis::GetHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked get_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_get_hypothesis(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_hypothesis", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_hypotheses", description = "List all hypotheses with optional filters.")]
async fn list_hypotheses(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ListHypothesesInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_hypotheses").await {
        tracing::warn!("Workflow enforcement blocked list_hypotheses: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_list_hypotheses(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_hypotheses", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_observations", description = "List recorded observations.")]
async fn list_observations(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ListObservationsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_observations").await {
        tracing::warn!("Workflow enforcement blocked list_observations: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_list_observations(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_observations", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "evaluate_hypothesis", description = "Evaluate a hypothesis based on its evidence.")]
async fn evaluate_hypothesis(
    &self,
    Parameters(input): Parameters<tools::hypothesis::EvaluateHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("evaluate_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked evaluate_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_evaluate_hypothesis(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("evaluate_hypothesis", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_evidence", description = "Get a specific evidence record by its ID.")]
async fn get_evidence(
    &self,
    Parameters(input): Parameters<tools::hypothesis::GetEvidenceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_evidence").await {
        tracing::warn!("Workflow enforcement blocked get_evidence: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_get_evidence(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_evidence", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_evidence", description = "List all evidence records across hypotheses.")]
async fn list_evidence(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ListEvidenceInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_evidence").await {
        tracing::warn!("Workflow enforcement blocked list_evidence: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_list_evidence(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("list_evidence", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}


#[tool(name = "get_knowledge", description = "Get learned knowledge extracted from validated hypotheses.")]
async fn get_knowledge(
    &self,
    Parameters(input): Parameters<tools::hypothesis::GetKnowledgeInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_knowledge").await {
        tracing::warn!("Workflow enforcement blocked get_knowledge: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_get_knowledge(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("get_knowledge", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "extract_knowledge", description = "Extract knowledge from a validated hypothesis.")]
async fn extract_knowledge(
    &self,
    Parameters(input): Parameters<tools::hypothesis::ExtractKnowledgeInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("extract_knowledge").await {
        tracing::warn!("Workflow enforcement blocked extract_knowledge: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::hypothesis::execute_extract_knowledge(input, &self.context.database).await {
        Ok(result) => {
            self.record_tool_execution("extract_knowledge", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "add_knowledge", description = "Add new validated knowledge to the knowledge base")]
async fn add_knowledge(
    &self,
    Parameters(input): Parameters<tools::knowledge::AddKnowledgeInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_knowledge").await {
        tracing::warn!("Workflow enforcement blocked add_knowledge: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::knowledge::execute_add_knowledge(input, &self.context.knowledge).await;
    if result.success {
        self.record_tool_execution("add_knowledge", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "query_knowledge", description = "Query the knowledge base for relevant knowledge")]
async fn query_knowledge(
    &self,
    Parameters(input): Parameters<tools::knowledge::QueryKnowledgeInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("query_knowledge").await {
        tracing::warn!("Workflow enforcement blocked query_knowledge: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::knowledge::execute_query_knowledge(input, &self.context.knowledge).await;
    if result.success {
        self.record_tool_execution("query_knowledge", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "record_knowledge_application", description = "Record the result of applying knowledge")]
async fn record_knowledge_application(
    &self,
    Parameters(input): Parameters<tools::knowledge::RecordKnowledgeApplicationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("record_knowledge_application").await {
        tracing::warn!("Workflow enforcement blocked record_knowledge_application: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::knowledge::execute_record_knowledge_application(input, &self.context.knowledge).await;
    if result.success {
        self.record_tool_execution("record_knowledge_application", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "get_knowledge_stats", description = "Get statistics about the knowledge base")]
async fn get_knowledge_stats(
    &self,
    Parameters(input): Parameters<tools::knowledge::GetKnowledgeStatsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_knowledge_stats").await {
        tracing::warn!("Workflow enforcement blocked get_knowledge_stats: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::knowledge::execute_get_knowledge_stats(input, &self.context.knowledge).await;
    if result.success {
        self.record_tool_execution("get_knowledge_stats", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "get_mature_knowledge", description = "Get all mature (high-confidence) knowledge")]
async fn get_mature_knowledge(
    &self,
    Parameters(input): Parameters<tools::knowledge::GetMatureKnowledgeInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_mature_knowledge").await {
        tracing::warn!("Workflow enforcement blocked get_mature_knowledge: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::knowledge::execute_get_mature_knowledge(input, &self.context.knowledge).await;
    if result.success {
        self.record_tool_execution("get_mature_knowledge", None).await;
    }
    tool_output_to_content(result)
}


#[tool(name = "create_plan", description = "Create a new plan from a goal")]
async fn create_plan(
    &self,
    Parameters(input): Parameters<tools::planner::CreatePlanInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("create_plan").await {
        tracing::warn!("Workflow enforcement blocked create_plan: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_create_plan(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("create_plan", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "add_plan_step", description = "Add a step to an existing plan")]
async fn add_plan_step(
    &self,
    Parameters(input): Parameters<tools::planner::AddPlanStepInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_plan_step").await {
        tracing::warn!("Workflow enforcement blocked add_plan_step: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_add_plan_step(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("add_plan_step", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "add_step_dependency", description = "Add a dependency between steps")]
async fn add_step_dependency(
    &self,
    Parameters(input): Parameters<tools::planner::AddStepDependencyInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_step_dependency").await {
        tracing::warn!("Workflow enforcement blocked add_step_dependency: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_add_step_dependency(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("add_step_dependency", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "get_plan", description = "Get a plan by ID")]
async fn get_plan(
    &self,
    Parameters(input): Parameters<tools::planner::GetPlanInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_plan").await {
        tracing::warn!("Workflow enforcement blocked get_plan: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_get_plan(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("get_plan", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "list_plans", description = "List all active plans")]
async fn list_plans(
    &self,
    Parameters(input): Parameters<tools::planner::ListPlansInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_plans").await {
        tracing::warn!("Workflow enforcement blocked list_plans: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_list_plans(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("list_plans", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "start_plan", description = "Start executing a plan")]
async fn start_plan(
    &self,
    Parameters(input): Parameters<tools::planner::StartPlanInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("start_plan").await {
        tracing::warn!("Workflow enforcement blocked start_plan: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_start_plan(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("start_plan", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "complete_step", description = "Mark a step as completed")]
async fn complete_step(
    &self,
    Parameters(input): Parameters<tools::planner::CompleteStepInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("complete_step").await {
        tracing::warn!("Workflow enforcement blocked complete_step: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_complete_step(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("complete_step", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "fail_step", description = "Mark a step as failed")]
async fn fail_step(
    &self,
    Parameters(input): Parameters<tools::planner::FailStepInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("fail_step").await {
        tracing::warn!("Workflow enforcement blocked fail_step: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_fail_step(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("fail_step", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "cancel_plan", description = "Cancel a plan")]
async fn cancel_plan(
    &self,
    Parameters(input): Parameters<tools::planner::CancelPlanInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("cancel_plan").await {
        tracing::warn!("Workflow enforcement blocked cancel_plan: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::planner::execute_cancel_plan(input, &self.context.planner).await;
    if result.success {
        self.record_tool_execution("cancel_plan", None).await;
    }
    tool_output_to_content(result)
}


#[tool(name = "create_workflow", description = "Create a new workflow with a name and optional description")]
async fn create_workflow(
    &self,
    Parameters(input): Parameters<tools::workflow::CreateWorkflowInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("create_workflow").await {
        tracing::warn!("Workflow enforcement blocked create_workflow: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_create_workflow(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("create_workflow", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "add_workflow_step", description = "Add a step to an existing workflow.")]
async fn add_workflow_step(
    &self,
    Parameters(input): Parameters<tools::workflow::AddWorkflowStepInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_workflow_step").await {
        tracing::warn!("Workflow enforcement blocked add_workflow_step: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_add_workflow_step(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("add_workflow_step", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "get_workflow_status", description = "Get the current status and details of a workflow")]
async fn get_workflow_status(
    &self,
    Parameters(input): Parameters<tools::workflow::GetWorkflowStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_workflow_status").await {
        tracing::warn!("Workflow enforcement blocked get_workflow_status: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_get_workflow_status(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("get_workflow_status", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "list_workflows", description = "List all workflows, optionally filtered by status")]
async fn list_workflows(
    &self,
    Parameters(input): Parameters<tools::workflow::ListWorkflowsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_workflows").await {
        tracing::warn!("Workflow enforcement blocked list_workflows: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_list_workflows(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("list_workflows", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "start_workflow", description = "Start executing a workflow.")]
async fn start_workflow(
    &self,
    Parameters(input): Parameters<tools::workflow::StartWorkflowInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("start_workflow").await {
        tracing::warn!("Workflow enforcement blocked start_workflow: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_start_workflow(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("start_workflow", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "pause_workflow", description = "Pause a running workflow")]
async fn pause_workflow(
    &self,
    Parameters(input): Parameters<tools::workflow::PauseWorkflowInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("pause_workflow").await {
        tracing::warn!("Workflow enforcement blocked pause_workflow: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_pause_workflow(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("pause_workflow", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "resume_workflow", description = "Resume a paused workflow")]
async fn resume_workflow(
    &self,
    Parameters(input): Parameters<tools::workflow::ResumeWorkflowInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("resume_workflow").await {
        tracing::warn!("Workflow enforcement blocked resume_workflow: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_resume_workflow(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("resume_workflow", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "cancel_workflow", description = "Cancel a workflow, removing it from execution.")]
async fn cancel_workflow(
    &self,
    Parameters(input): Parameters<tools::workflow::CancelWorkflowInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("cancel_workflow").await {
        tracing::warn!("Workflow enforcement blocked cancel_workflow: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_cancel_workflow(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("cancel_workflow", None).await;
    }
    tool_output_to_content(result)
}

#[tool(name = "delete_workflow", description = "Delete a workflow completely.")]
async fn delete_workflow(
    &self,
    Parameters(input): Parameters<tools::workflow::DeleteWorkflowInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("delete_workflow").await {
        tracing::warn!("Workflow enforcement blocked delete_workflow: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::workflow::execute_delete_workflow(input, &self.context.workflow_engine).await;
    if result.success {
        self.record_tool_execution("delete_workflow", None).await;
    }
    tool_output_to_content(result)
}


#[tool(
    name = "start_exploration",
    description = "Start a new exploration session. Explorations allow RoBoT to actively investigate topics and test hypotheses."
)]
async fn start_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::StartExplorationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("start_exploration").await {
        tracing::warn!("Workflow enforcement blocked start_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_start_exploration(input);
    if result.success {
        self.record_tool_execution("start_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "get_exploration_status",
    description = "Get the current status of an exploration including hypotheses, attempts, and findings."
)]
async fn get_exploration_status(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_exploration_status").await {
        tracing::warn!("Workflow enforcement blocked get_exploration_status: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_get_exploration_status(input);
    if result.success {
        self.record_tool_execution("get_exploration_status", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "complete_exploration",
    description = "Mark an exploration as completed with findings."
)]
async fn complete_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::CompleteExplorationInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("complete_exploration").await {
        tracing::warn!("Workflow enforcement blocked complete_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_complete_exploration(input);
    if result.success {
        self.record_tool_execution("complete_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "abandon_exploration",
    description = "Abandon an exploration without completing it."
)]
async fn abandon_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("abandon_exploration").await {
        tracing::warn!("Workflow enforcement blocked abandon_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_abandon_exploration(input);
    if result.success {
        self.record_tool_execution("abandon_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "record_attempt",
    description = "Record an attempt made during exploration."
)]
async fn record_attempt(
    &self,
    Parameters(input): Parameters<tools::exploration::RecordAttemptInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("record_attempt").await {
        tracing::warn!("Workflow enforcement blocked record_attempt: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_record_attempt(input);
    if result.success {
        self.record_tool_execution("record_attempt", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "add_exploration_hypothesis",
    description = "Add a testable hypothesis to an exploration."
)]
async fn add_exploration_hypothesis(
    &self,
    Parameters(input): Parameters<tools::exploration::AddHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("add_exploration_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked add_exploration_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_add_hypothesis(input);
    if result.success {
        self.record_tool_execution("add_exploration_hypothesis", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "evaluate_exploration_hypothesis",
    description = "Set the result for a hypothesis based on evidence."
)]
async fn evaluate_exploration_hypothesis(
    &self,
    Parameters(input): Parameters<tools::exploration::EvaluateHypothesisInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("evaluate_exploration_hypothesis").await {
        tracing::warn!("Workflow enforcement blocked evaluate_exploration_hypothesis: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_evaluate_hypothesis(input);
    if result.success {
        self.record_tool_execution("evaluate_exploration_hypothesis", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "promote_finding",
    description = "Promote a finding from an exploration to reusable knowledge."
)]
async fn promote_finding(
    &self,
    Parameters(input): Parameters<tools::exploration::PromoteFindingInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("promote_finding").await {
        tracing::warn!("Workflow enforcement blocked promote_finding: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_promote_finding(input);
    if result.success {
        self.record_tool_execution("promote_finding", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "pause_exploration",
    description = "Pause an active exploration."
)]
async fn pause_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("pause_exploration").await {
        tracing::warn!("Workflow enforcement blocked pause_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_pause_exploration(input);
    if result.success {
        self.record_tool_execution("pause_exploration", None).await;
    }
    tool_output_to_content(result)
}

#[tool(
    name = "resume_exploration",
    description = "Resume a paused exploration."
)]
async fn resume_exploration(
    &self,
    Parameters(input): Parameters<tools::exploration::GetExplorationStatusInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("resume_exploration").await {
        tracing::warn!("Workflow enforcement blocked resume_exploration: {}", e.message);
        return enforcement_error_to_content(e);
    }
    let result = tools::exploration::execute_resume_exploration(input);
    if result.success {
        self.record_tool_execution("resume_exploration", None).await;
    }
    tool_output_to_content(result)
}


#[tool(name = "register_skill", description = "Register a new skill in the skill registry.")]
async fn register_skill(
    &self,
    Parameters(input): Parameters<tools::skills::RegisterSkillInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("register_skill").await {
        tracing::warn!("Workflow enforcement blocked register_skill: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_register_skill(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("register_skill", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "discover_skill", description = "Create a skill discovered from an experience.")]
async fn discover_skill(
    &self,
    Parameters(input): Parameters<tools::skills::DiscoverSkillInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("discover_skill").await {
        tracing::warn!("Workflow enforcement blocked discover_skill: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_discover_skill(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("discover_skill", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_skill", description = "Get details of a specific skill including mastery level.")]
async fn get_skill(
    &self,
    Parameters(input): Parameters<tools::skills::GetSkillInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_skill").await {
        tracing::warn!("Workflow enforcement blocked get_skill: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_get_skill(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("get_skill", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "list_skills", description = "List all registered skills, optionally filtered.")]
async fn list_skills(
    &self,
    Parameters(input): Parameters<tools::skills::ListSkillsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("list_skills").await {
        tracing::warn!("Workflow enforcement blocked list_skills: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_list_skills(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("list_skills", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "update_skill_mastery", description = "Update skill mastery based on execution outcome.")]
async fn update_skill_mastery(
    &self,
    Parameters(input): Parameters<tools::skills::UpdateSkillMasteryInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("update_skill_mastery").await {
        tracing::warn!("Workflow enforcement blocked update_skill_mastery: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_update_skill_mastery(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("update_skill_mastery", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_skill_recommendations", description = "Get skill recommendations based on readiness.")]
async fn get_skill_recommendations(
    &self,
    Parameters(input): Parameters<tools::skills::GetSkillRecommendationsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_skill_recommendations").await {
        tracing::warn!("Workflow enforcement blocked get_skill_recommendations: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_get_skill_recommendations(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("get_skill_recommendations", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "execute_skill", description = "Execute a skill with provided task and parameters.")]
async fn execute_skill(
    &self,
    Parameters(input): Parameters<tools::skills::ExecuteSkillInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("execute_skill").await {
        tracing::warn!("Workflow enforcement blocked execute_skill: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_execute_skill(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("execute_skill", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "get_skill_stats", description = "Get comprehensive statistics about the skill registry.")]
async fn get_skill_stats(
    &self,
    Parameters(input): Parameters<tools::skills::GetSkillStatsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("get_skill_stats").await {
        tracing::warn!("Workflow enforcement blocked get_skill_stats: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_get_skill_stats(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("get_skill_stats", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "apply_skill_decay", description = "Apply mastery decay to unused skills.")]
async fn apply_skill_decay(
    &self,
    Parameters(input): Parameters<tools::skills::ApplySkillDecayInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("apply_skill_decay").await {
        tracing::warn!("Workflow enforcement blocked apply_skill_decay: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_apply_skill_decay(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("apply_skill_decay", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "enable_disable_skill", description = "Enable or disable a skill.")]
async fn enable_disable_skill(
    &self,
    Parameters(input): Parameters<tools::skills::EnableDisableSkillInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("enable_disable_skill").await {
        tracing::warn!("Workflow enforcement blocked enable_disable_skill: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_enable_disable_skill(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("enable_disable_skill", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

#[tool(name = "search_skills", description = "Search skills by query, category, or minimum mastery level.")]
async fn search_skills(
    &self,
    Parameters(input): Parameters<tools::skills::SearchSkillsInput>,
) -> ContentBlock {
    if let Err(e) = self.check_workflow_enforcement("search_skills").await {
        tracing::warn!("Workflow enforcement blocked search_skills: {}", e.message);
        return enforcement_error_to_content(e);
    }
    match tools::skills::execute_search_skills(input, &self.context).await {
        Ok(result) => {
            self.record_tool_execution("search_skills", None).await;
            tool_output_to_content(result)
        }
        Err(e) => tool_output_to_content(ToolOutput::error(e)),
    }
}

}

