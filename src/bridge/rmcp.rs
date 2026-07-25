// src/bridge/rmcp.rs
// RMCP (Rust MCP) server implementation using the rmcp crate

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    handler::server::ServerHandler,
    model::{ContentBlock, Implementation, ServerInfo, TextContent},
    serve_server, tool, tool_handler, tool_router,
};

use super::mcp::McpContext;
use crate::tools::{self, ToolOutput};
use crate::workflows::enforcement::{WorkflowEnforcer, WorkflowEnforcementError};

/// Convert ToolOutput to MCP-compliant ContentBlock
///
/// MCP protocol requires tool responses to have a `content` array with text/image/audio blocks:
/// ```json
/// {
///   "content": [
///     {"type": "text", "text": "..."}
///   ]
/// }
/// ```
fn tool_output_to_content(output: ToolOutput) -> ContentBlock {
    let text = if output.success {
        serde_json::to_string_pretty(&output.data)
            .unwrap_or_else(|_| r#"{"success": true}"#.to_string())
    } else {
        // Always return JSON for errors - MCP clients need to parse the response
        serde_json::to_string_pretty(&serde_json::json!({
            "success": false,
            "error": output.error.unwrap_or_else(|| "Unknown error".to_string())
        }))
        .unwrap_or_else(|_| r#"{"success": false, "error": "Failed to serialize error"}"#.to_string())
    };

    ContentBlock::Text(TextContent::new(text))
}

/// RMCP server wrapper for MCP bridge (reserved for future use)
#[allow(dead_code)]
pub struct RmcpServer {
    context: Arc<McpContext>,
}

#[allow(dead_code)]
impl RmcpServer {
    /// Get the shared context
    pub fn context(&self) -> Arc<McpContext> {
        Arc::clone(&self.context)
    }
}

/// Create a new RMCP server with stdio transport
pub async fn run_stdio_server(name: &str, version: &str, context: Arc<McpContext>) -> Result<()> {
    tracing::info!(
        "Starting RMCP server '{}' v{} with stdio transport",
        name,
        version
    );

    let enforcer = Arc::new(WorkflowEnforcer::new());
    let session_counter = Arc::new(AtomicU64::new(1));

    let handler = McpServerHandler {
        context,
        name: name.to_string(),
        version: version.to_string(),
        enforcer,
        session_counter,
        session_id: "default".to_string(),
    };

    // Log the tools that will be exposed
    let router = McpServerHandler::tool_router();
    let tools = router.list_all();
    tracing::info!("MCP tools exposed via rmcp: {} tools", tools.len());
    for tool in &tools {
        tracing::debug!("  - {}: {:?}", tool.name, tool.description);
    }

    // Use tokio's stdin/stdout - this should work on all platforms
    // On Windows, tokio handles the complexity of async IO with subprocess pipes
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    // For debugging: print a marker to stderr so we can see startup
    eprintln!("DEBUG: About to call serve_server");

    // Start the server - this will block waiting for MCP messages
    let running = serve_server(handler, (stdin, stdout)).await?;

    eprintln!("DEBUG: serve_server returned");
    eprintln!("DEBUG: Server is now listening for messages...");

    tracing::info!("Server started, waiting for connections...");

    // Wait for the service to complete (until transport closes)
    let quit_reason = running.waiting().await?;

    tracing::info!("Server stopped: {:?}", quit_reason);

    Ok(())
}

/// Helper function to convert enforcement error to ContentBlock
fn enforcement_error_to_content(error: WorkflowEnforcementError) -> ContentBlock {
    let text = serde_json::to_string_pretty(&serde_json::json!({
        "success": false,
        "error": {
            "code": error.error_code,
            "message": error.message,
            "required_action": error.required_action,
            "blocked_tools": error.tools_blocked
        },
        "hint": "Call get_workflow first, then search_memory before using other tools."
    }))
    .unwrap_or_else(|_| r#"{"success": false, "error": "Enforcement error"}"#.to_string());

    ContentBlock::Text(TextContent::new(text))
}

/// MCP Server handler using the rmcp derive macros
#[derive(Clone)]
struct McpServerHandler {
    context: Arc<McpContext>,
    name: String,
    version: String,
    enforcer: Arc<WorkflowEnforcer>,
    #[allow(dead_code)]
    session_counter: Arc<AtomicU64>,
    session_id: String,
}

impl McpServerHandler {
    #[allow(dead_code)]
    fn new(context: Arc<McpContext>, name: String, version: String) -> Self {
        Self {
            context,
            name,
            version,
            enforcer: Arc::new(WorkflowEnforcer::new()),
            session_counter: Arc::new(AtomicU64::new(1)),
            session_id: "default".to_string(),
        }
    }

    /// Generate a new session ID
    #[allow(dead_code)]
    fn new_session(&self) -> String {
        let id = self.session_counter.fetch_add(1, Ordering::SeqCst);
        format!("session-{}", id)
    }

    /// Check workflow enforcement before executing a tool
    async fn check_workflow_enforcement(&self, tool_name: &str) -> Result<(), WorkflowEnforcementError> {
        self.enforcer.check_enforcement(&self.session_id, tool_name).await
    }

    /// Record successful tool execution
    async fn record_tool_execution(&self, tool_name: &str, query: Option<String>) {
        self.enforcer.record_tool_execution(&self.session_id, tool_name, query).await;
    }
}

#[tool_router]
impl McpServerHandler {
    #[tool(
        name = "get_workflow",
        description = "MANDATORY: Get workflow rules. MUST be called before any other tool. Returns the required workflow for this MCP server."
    )]
    async fn get_workflow(
        &self,
        Parameters(input): Parameters<tools::agent::GetWorkflowInput>,
    ) -> ContentBlock {
        // Record workflow retrieval
        self.enforcer.record_workflow_retrieved(&self.session_id, input.purpose.clone()).await;
        
        match tools::agent::execute_get_workflow(input).await {
            Ok(result) => {
                self.record_tool_execution("get_workflow", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "store_memory",
        description = "Store a new memory in the knowledge base"
    )]
    async fn store_memory(
        &self,
        Parameters(input): Parameters<tools::memory::StoreMemoryInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("store_memory").await {
            tracing::warn!("Workflow enforcement blocked store_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::memory::execute_store_memory(input, &self.context.database).await {
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
        // Check workflow enforcement first (but search_memory is allowed after get_workflow)
        if let Err(e) = self.check_workflow_enforcement("search_memory").await {
            tracing::warn!("Workflow enforcement blocked search_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let query = Some(input.query.clone());
        match tools::memory::execute_search_memory(input, &self.context.database).await {
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
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_memory").await {
            tracing::warn!("Workflow enforcement blocked get_memory: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::memory::execute_get_memory(input, &self.context.database).await {
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
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("list_memories").await {
            tracing::warn!("Workflow enforcement blocked list_memories: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::memory::execute_list_memories(input, &self.context.database).await {
            Ok(result) => {
                self.record_tool_execution("list_memories", None).await;
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
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("record_experience").await {
            tracing::warn!("Workflow enforcement blocked record_experience: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::experience::execute_record_experience(
            input,
            &self.context.coordinator,
            &self.context.database,
        )
        .await
        {
            Ok(result) => {
                self.record_tool_execution("record_experience", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "get_experience_stats",
        description = "Get experience statistics"
    )]
    async fn get_experience_stats(
        &self,
        Parameters(input): Parameters<tools::experience::GetExperienceStatsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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

    #[tool(
        name = "get_experience",
        description = "Get a specific experience by ID"
    )]
    async fn get_experience(
        &self,
        Parameters(input): Parameters<tools::experience::GetExperienceInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "get_insights",
        description = "Get actionable insights from reflections"
    )]
    async fn get_insights(
        &self,
        Parameters(input): Parameters<tools::reflection::GetInsightsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (but get_insights is allowed as memory search step)
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
        // Check workflow enforcement first
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

    #[tool(
        name = "analyze_patterns",
        description = "Analyze experiences to detect patterns"
    )]
    async fn analyze_patterns(
        &self,
        Parameters(input): Parameters<tools::reflection::AnalyzePatternsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (but analyze_patterns counts as memory search)
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
        // Check workflow enforcement first (but get_patterns counts as memory search)
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

    #[tool(
        name = "global_search",
        description = "Search across all memories and experiences"
    )]
    async fn global_search(
        &self,
        Parameters(input): Parameters<tools::search::GlobalSearchInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (but global_search counts as memory search)
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

    #[tool(
        name = "get_recommendations",
        description = "Get recommendations based on learned patterns"
    )]
    async fn get_recommendations(
        &self,
        Parameters(input): Parameters<tools::search::GetRecommendationsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "get_reputation",
        description = "Get reputation score for a target"
    )]
    async fn get_reputation(
        &self,
        Parameters(input): Parameters<tools::search::GetReputationInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
        description = "Ingest files from a folder into memory"
    )]
    async fn ingest_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::IngestFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("ingest_files").await {
            tracing::warn!("Workflow enforcement blocked ingest_files: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::ingest_file(input, Arc::clone(&self.context.database)).await {
            Ok(result) => {
                self.record_tool_execution("ingest_files", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "list_importable",
        description = "List files available for import"
    )]
    async fn list_importable(
        &self,
        Parameters(input): Parameters<tools::ingestor::ListImportableInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "transcribe_audio",
        description = "Transcribe an audio file to text"
    )]
    async fn transcribe_audio(
        &self,
        Parameters(input): Parameters<tools::ingestor::TranscribeAudioInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("transcribe_audio").await {
            tracing::warn!("Workflow enforcement blocked transcribe_audio: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        match tools::ingestor::execute_transcribe_audio(input).await {
            Ok(result) => {
                self.record_tool_execution("transcribe_audio", None).await;
                tool_output_to_content(result)
            }
            Err(e) => tool_output_to_content(ToolOutput::error(e)),
        }
    }

    #[tool(
        name = "list_ingested_files",
        description = "List files that have been ingested"
    )]
    async fn list_ingested_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::ListIngestedFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
        description = "Delete successfully ingested files"
    )]
    async fn delete_ingested_files(
        &self,
        Parameters(input): Parameters<tools::ingestor::DeleteIngestedFilesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first (destructive operation - requires full workflow)
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

    // ========================================================================
    // HYPOTHESIS ENGINE TOOLS
    // Observation -> Hypothesis -> Test -> Evidence -> Knowledge
    // ========================================================================

    #[tool(
        name = "record_observation",
        description = "Record an observation. Observations are the starting point for learning - record successes, failures, patterns, or anomalies."
    )]
    async fn record_observation(
        &self,
        Parameters(input): Parameters<tools::hypothesis::RecordObservationInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "create_hypothesis",
        description = "Create a testable hypothesis from observations."
    )]
    async fn create_hypothesis(
        &self,
        Parameters(input): Parameters<tools::hypothesis::CreateHypothesisInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "add_evidence",
        description = "Add evidence to a hypothesis. Evidence can support or contradict."
    )]
    async fn add_evidence(
        &self,
        Parameters(input): Parameters<tools::hypothesis::AddEvidenceInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "get_hypothesis",
        description = "Get details of a specific hypothesis including all its evidence."
    )]
    async fn get_hypothesis(
        &self,
        Parameters(input): Parameters<tools::hypothesis::GetHypothesisInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "list_hypotheses",
        description = "List all hypotheses with optional filters."
    )]
    async fn list_hypotheses(
        &self,
        Parameters(input): Parameters<tools::hypothesis::ListHypothesesInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "list_observations",
        description = "List recorded observations."
    )]
    async fn list_observations(
        &self,
        Parameters(input): Parameters<tools::hypothesis::ListObservationsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "evaluate_hypothesis",
        description = "Evaluate a hypothesis based on its evidence and update its status."
    )]
    async fn evaluate_hypothesis(
        &self,
        Parameters(input): Parameters<tools::hypothesis::EvaluateHypothesisInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "get_knowledge",
        description = "Get learned knowledge extracted from validated hypotheses."
    )]
    async fn get_knowledge(
        &self,
        Parameters(input): Parameters<tools::hypothesis::GetKnowledgeInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "extract_knowledge",
        description = "Extract knowledge from a validated hypothesis into reusable knowledge."
    )]
    async fn extract_knowledge(
        &self,
        Parameters(input): Parameters<tools::hypothesis::ExtractKnowledgeInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    // Knowledge tools
    #[tool(
        name = "add_knowledge",
        description = "Add new validated knowledge to the knowledge base"
    )]
    async fn add_knowledge(
        &self,
        Parameters(input): Parameters<tools::knowledge::AddKnowledgeInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "query_knowledge",
        description = "Query the knowledge base for relevant knowledge"
    )]
    async fn query_knowledge(
        &self,
        Parameters(input): Parameters<tools::knowledge::QueryKnowledgeInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "record_knowledge_application",
        description = "Record the result of applying knowledge"
    )]
    async fn record_knowledge_application(
        &self,
        Parameters(input): Parameters<tools::knowledge::RecordKnowledgeApplicationInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("record_knowledge_application").await {
            tracing::warn!("Workflow enforcement blocked record_knowledge_application: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::knowledge::execute_record_knowledge_application(input, &self.context.knowledge)
            .await;
        if result.success {
            self.record_tool_execution("record_knowledge_application", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(
        name = "get_knowledge_stats",
        description = "Get statistics about the knowledge base"
    )]
    async fn get_knowledge_stats(
        &self,
        Parameters(input): Parameters<tools::knowledge::GetKnowledgeStatsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "get_mature_knowledge",
        description = "Get all mature (high-confidence) knowledge"
    )]
    async fn get_mature_knowledge(
        &self,
        Parameters(input): Parameters<tools::knowledge::GetMatureKnowledgeInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    // Planner tools
    #[tool(name = "create_plan", description = "Create a new plan from a goal")]
    async fn create_plan(
        &self,
        Parameters(input): Parameters<tools::planner::CreatePlanInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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

    #[tool(
        name = "add_step_dependency",
        description = "Add a dependency between steps"
    )]
    async fn add_step_dependency(
        &self,
        Parameters(input): Parameters<tools::planner::AddStepDependencyInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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

    // ========================================================================
    // WORKFLOW ENGINE TOOLS
    // ========================================================================

    #[tool(
        name = "create_workflow",
        description = "Create a new workflow with a name and optional description"
    )]
    async fn create_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::CreateWorkflowInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "add_workflow_step",
        description = "Add a step to an existing workflow. Steps are executed in order."
    )]
    async fn add_workflow_step(
        &self,
        Parameters(input): Parameters<tools::workflow::AddWorkflowStepInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "get_workflow_status",
        description = "Get the current status and details of a workflow"
    )]
    async fn get_workflow_status(
        &self,
        Parameters(input): Parameters<tools::workflow::GetWorkflowStatusInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
        if let Err(e) = self.check_workflow_enforcement("get_workflow_status").await {
            tracing::warn!("Workflow enforcement blocked get_workflow_status: {}", e.message);
            return enforcement_error_to_content(e);
        }
        
        let result = tools::workflow::execute_get_workflow_status(input, &self.context.workflow_engine)
            .await;
        if result.success {
            self.record_tool_execution("get_workflow_status", None).await;
        }
        tool_output_to_content(result)
    }

    #[tool(
        name = "list_workflows",
        description = "List all workflows, optionally filtered by status"
    )]
    async fn list_workflows(
        &self,
        Parameters(input): Parameters<tools::workflow::ListWorkflowsInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "start_workflow",
        description = "Start executing a workflow. The engine will run all steps sequentially."
    )]
    async fn start_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::StartWorkflowInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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
        // Check workflow enforcement first
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

    #[tool(
        name = "cancel_workflow",
        description = "Cancel a workflow, removing it from execution."
    )]
    async fn cancel_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::CancelWorkflowInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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

    #[tool(
        name = "delete_workflow",
        description = "Delete a workflow completely."
    )]
    async fn delete_workflow(
        &self,
        Parameters(input): Parameters<tools::workflow::DeleteWorkflowInput>,
    ) -> ContentBlock {
        // Check workflow enforcement first
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
}

// Manual ServerHandler impl with custom server info
#[tool_handler]
impl ServerHandler for McpServerHandler {
    fn get_info(&self) -> ServerInfo {
        // Create server capabilities using builder - MUST include tools to be recognized by MCP clients
        use rmcp::model::ServerCapabilitiesBuilder;

        // Builder order matters: each enable_ requires previous ones to be enabled
        #[allow(deprecated)]
        let capabilities = ServerCapabilitiesBuilder::default()
            .enable_experimental()
            .enable_extensions()
            .enable_logging()
            .enable_completions()
            .enable_prompts()
            .enable_resources()
            .enable_tasks()
            .enable_tools()
            .enable_tool_list_changed()
            .build();

        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new(&self.name, &self.version))
    }
}
