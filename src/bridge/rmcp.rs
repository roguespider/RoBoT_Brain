// src/bridge/rmcp.rs
// RMCP (Rust MCP) server implementation using the rmcp crate

use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    serve_server,
    tool_router,
    tool,
    handler::server::wrapper::{Parameters, Json},
};

use super::mcp::McpContext;
use crate::tools;

/// RMCP server wrapper for MCP bridge
pub struct RmcpServer {
    context: Arc<McpContext>,
}

impl RmcpServer {
    /// Get the shared context
    pub fn context(&self) -> Arc<McpContext> {
        Arc::clone(&self.context)
    }
}

/// Create a new RMCP server with stdio transport
pub async fn run_stdio_server(
    name: &str,
    version: &str,
    context: Arc<McpContext>,
) -> Result<()> {
    tracing::info!("Starting RMCP server '{}' v{} with stdio transport", name, version);
    
    let handler = McpServerHandler {
        context,
        name: name.to_string(),
        version: version.to_string(),
    };
    
    // Debug: Log the tools that will be exposed
    let router = McpServerHandler::tool_router();
    let tools = router.list_all();
    tracing::info!("MCP tools exposed via rmcp: {} tools", tools.len());
    for tool in &tools {
        tracing::debug!("  - {}: {:?}", tool.name, tool.description);
    }
    
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    
    serve_server(handler, (stdin, stdout)).await?;
    
    Ok(())
}

/// MCP Server handler using the rmcp derive macros
#[derive(Clone)]
struct McpServerHandler {
    context: Arc<McpContext>,
    name: String,
    version: String,
}

impl McpServerHandler {
    fn new(context: Arc<McpContext>, name: String, version: String) -> Self {
        Self { context, name, version }
    }
}

#[tool_router(server_handler)]
impl McpServerHandler {
    /// Ping handler for connection verification
    #[tool(name = "ping", description = "Ping the MCP server to verify connection")]
    async fn ping(
        &self,
        Parameters(input): Parameters<tools::agent::PingInput>,
    ) -> Json<serde_json::Value> {
        match tools::agent::execute_ping(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "store_memory", description = "Store a new memory in the knowledge base")]
    async fn store_memory(
        &self,
        Parameters(input): Parameters<tools::memory::StoreMemoryInput>,
    ) -> Json<serde_json::Value> {
        match tools::memory::execute_store_memory(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "search_memory", description = "Search memories by content")]
    async fn search_memory(
        &self,
        Parameters(input): Parameters<tools::memory::SearchMemoryInput>,
    ) -> Json<serde_json::Value> {
        match tools::memory::execute_search_memory(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_memory", description = "Get a specific memory by ID")]
    async fn get_memory(
        &self,
        Parameters(input): Parameters<tools::memory::GetMemoryInput>,
    ) -> Json<serde_json::Value> {
        match tools::memory::execute_get_memory(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "list_memories", description = "List recent memories")]
    async fn list_memories(
        &self,
        Parameters(input): Parameters<tools::memory::ListMemoriesInput>,
    ) -> Json<serde_json::Value> {
        match tools::memory::execute_list_memories(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "record_experience", description = "Record a new experience")]
    async fn record_experience(
        &self,
        Parameters(input): Parameters<tools::experience::RecordExperienceInput>,
    ) -> Json<serde_json::Value> {
        match tools::experience::execute_record_experience(
            input,
            &self.context.coordinator,
            &self.context.database,
        ).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_experience_stats", description = "Get experience statistics")]
    async fn get_experience_stats(
        &self,
        Parameters(input): Parameters<tools::experience::GetExperienceStatsInput>,
    ) -> Json<serde_json::Value> {
        match tools::experience::execute_get_experience_stats(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "list_experiences", description = "List recent experiences")]
    async fn list_experiences(
        &self,
        Parameters(input): Parameters<tools::experience::ListExperiencesInput>,
    ) -> Json<serde_json::Value> {
        match tools::experience::execute_list_experiences(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_experience", description = "Get a specific experience by ID")]
    async fn get_experience(
        &self,
        Parameters(input): Parameters<tools::experience::GetExperienceInput>,
    ) -> Json<serde_json::Value> {
        match tools::experience::execute_get_experience(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_insights", description = "Get actionable insights from reflections")]
    async fn get_insights(
        &self,
        Parameters(input): Parameters<tools::reflection::GetInsightsInput>,
    ) -> Json<serde_json::Value> {
        match tools::reflection::execute_get_insights(input, &self.context.reflection).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "create_reflection", description = "Create a new reflection")]
    async fn create_reflection(
        &self,
        Parameters(input): Parameters<tools::reflection::CreateReflectionInput>,
    ) -> Json<serde_json::Value> {
        match tools::reflection::execute_create_reflection(input, &self.context.reflection).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "analyze_patterns", description = "Analyze experiences to detect patterns")]
    async fn analyze_patterns(
        &self,
        Parameters(input): Parameters<tools::reflection::AnalyzePatternsInput>,
    ) -> Json<serde_json::Value> {
        match tools::reflection::execute_analyze_patterns(input, &self.context.reflection).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_patterns", description = "Get detected patterns")]
    async fn get_patterns(
        &self,
        Parameters(input): Parameters<tools::reflection::GetPatternsInput>,
    ) -> Json<serde_json::Value> {
        match tools::reflection::execute_get_patterns(input, &self.context.reflection).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "global_search", description = "Search across all memories and experiences")]
    async fn global_search(
        &self,
        Parameters(input): Parameters<tools::search::GlobalSearchInput>,
    ) -> Json<serde_json::Value> {
        match tools::search::execute_global_search(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_recommendations", description = "Get recommendations based on learned patterns")]
    async fn get_recommendations(
        &self,
        Parameters(input): Parameters<tools::search::GetRecommendationsInput>,
    ) -> Json<serde_json::Value> {
        match tools::search::execute_get_recommendations(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_reputation", description = "Get reputation score for a target")]
    async fn get_reputation(
        &self,
        Parameters(input): Parameters<tools::search::GetReputationInput>,
    ) -> Json<serde_json::Value> {
        match tools::search::execute_get_reputation(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "ingest_files", description = "Ingest files from a folder into memory")]
    async fn ingest_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::IngestFilesInput>,
    ) -> Json<serde_json::Value> {
        match tools::ingestor::execute_ingest_files(input, &self.context.database).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "list_importable", description = "List files available for import")]
    async fn list_importable(
        &self,
        Parameters(input): Parameters<tools::ingestor::ListImportableInput>,
    ) -> Json<serde_json::Value> {
        match tools::ingestor::execute_list_importable(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "transcribe_audio", description = "Transcribe an audio file to text")]
    async fn transcribe_audio(
        &self,
        Parameters(input): Parameters<tools::ingestor::TranscribeAudioInput>,
    ) -> Json<serde_json::Value> {
        match tools::ingestor::execute_transcribe_audio(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "list_ingested_files", description = "List files that have been ingested")]
    async fn list_ingested_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::ListIngestedFilesInput>,
    ) -> Json<serde_json::Value> {
        match tools::ingestor::execute_list_ingested_files(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "delete_ingested_files", description = "Delete successfully ingested files")]
    async fn delete_ingested_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::DeleteIngestedFilesInput>,
    ) -> Json<serde_json::Value> {
        match tools::ingestor::execute_delete_ingested_files(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "mcp_list_tools", description = "List all available MCP tools with optional filter")]
    async fn mcp_list_tools(
        &self,
        Parameters(input): Parameters<tools::agent::ListToolsInput>,
    ) -> Json<serde_json::Value> {
        match tools::agent::execute_list_tools(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }

    #[tool(name = "get_tool", description = "Get detailed information about a specific tool")]
    async fn get_tool(
        &self,
        Parameters(input): Parameters<tools::agent::GetToolInput>,
    ) -> Json<serde_json::Value> {
        match tools::agent::execute_get_tool(input).await {
            Ok(result) => Json(result),
            Err(e) => Json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    }
}
