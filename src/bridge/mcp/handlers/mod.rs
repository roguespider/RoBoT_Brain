// src/bridge/mcp/handlers/mod.rs
// MCP tool handlers - HOW tools respond to MCP protocol
//
// Architecture:
// - This module contains ONLY MCP protocol handlers
// - Each handler implements ToolHandler trait for its local type
// - McpServerHandler aggregates all handlers via the ToolHandler trait
// - Graceful degradation: if a handler fails to init, log warning but continue
//
// Separation of concerns:
// - tools/ contains WHAT tools exist (definitions, schemas)
// - mcp/handlers/ contains HOW tools respond to MCP (execution logic)

use std::sync::Arc;

pub mod acp_handler;
pub mod agent_handler;
pub mod experience_handler;
pub mod exploration_handler;
pub mod hypothesis_handler;
pub mod ingestor_handler;
pub mod knowledge_handler;
pub mod memory_handler;
pub mod planner_handler;
pub mod reflection_handler;
pub mod search_handler;
pub mod skills_handler;
pub mod workflow_handler;

/// Result of attempting to initialize a tool handler
pub type HandlerInitResult<T> = Result<T, HandlerInitError>;

/// Error during handler initialization
#[derive(Debug, Clone)]
pub struct HandlerInitError {
    pub category: String,
    pub message: String,
}

impl HandlerInitError {
    pub fn new(category: &str, message: &str) -> Self {
        Self {
            category: category.to_string(),
            message: message.to_string(),
        }
    }
}

/// Error when executing a tool via MCP
#[derive(Debug, Clone)]
pub enum HandlerError {
    /// Tool was not found
    ToolNotFound(String),
    /// Handler for category not found
    HandlerNotFound(String),
    /// Tool execution failed
    ExecutionFailed(String),
    /// Invalid parameters
    InvalidParams(String),
    /// Internal handler error
    Internal(String),
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            HandlerError::HandlerNotFound(cat) => write!(f, "Handler not found for category: {}", cat),
            HandlerError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            HandlerError::InvalidParams(msg) => write!(f, "Invalid parameters: {}", msg),
            HandlerError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for HandlerError {}

/// Trait for tool handlers - each handler manages a category of tools
/// 
/// This trait allows McpServerHandler to aggregate all tool handlers
/// while keeping them isolated. If one handler fails, others continue.
pub trait ToolHandler: Send + Sync {
    /// Get the category name for this handler
    fn category(&self) -> &str;
    
    /// Get the list of tool names this handler manages
    fn tool_names(&self) -> Vec<String>;
    
    /// Check if this handler is healthy (can process requests)
    fn is_healthy(&self) -> bool;
    
    /// Get all tools this handler manages as MCP Tool definitions
    /// 
    /// Default implementation returns an empty vector.
    /// Override this method to provide actual tool definitions.
    fn get_tools(&self) -> Vec<rmcp::model::Tool> {
        Vec::new()
    }
    
    /// Execute a tool by name with arguments
    /// 
    /// Default implementation returns ToolNotFound error.
    /// Override this method to handle actual tool execution.
    fn execute_tool(&self, name: &str, _: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send {
        async move {
            Err(HandlerError::ToolNotFound(name.to_string()))
        }
    }
}

/// Marker trait for handlers that need workflow enforcement
pub trait WorkflowEnforced {
    fn check_enforcement(&self, tool_name: &str) -> impl std::future::Future<Output = Result<(), String>> + Send;
    fn record_execution(&self, tool_name: &str, query: Option<String>) -> impl std::future::Future<Output = ()> + Send;
}

/// Convert serde_json::Value to Arc<serde_json::Map<String, serde_json::Value>>
/// for use in Tool::new()
pub fn json_to_schema(schema: serde_json::Value) -> std::sync::Arc<serde_json::Map<String, serde_json::Value>> {
    match schema {
        serde_json::Value::Object(map) => std::sync::Arc::new(map),
        other => {
            let mut map = serde_json::Map::new();
            if let serde_json::Value::String(s) = other {
                map.insert("type".to_string(), serde_json::Value::String(s));
            }
            std::sync::Arc::new(map)
        }
    }
}

pub use acp_handler::AcpToolsHandler;
pub use agent_handler::AgentToolsHandler;
pub use experience_handler::ExperienceToolsHandler;
pub use exploration_handler::ExplorationToolsHandler;
pub use hypothesis_handler::HypothesisToolsHandler;
pub use ingestor_handler::IngestorToolsHandler;
pub use knowledge_handler::KnowledgeToolsHandler;
pub use memory_handler::MemoryToolsHandler;
pub use planner_handler::PlannerToolsHandler;
pub use reflection_handler::ReflectionToolsHandler;
pub use search_handler::SearchToolsHandler;
pub use skills_handler::SkillsToolsHandler;
pub use workflow_handler::WorkflowToolsHandler;

/// Collection of all tool handlers with graceful degradation
#[derive(Clone)]
pub struct ToolHandlerCollection {
    pub acp: Option<AcpToolsHandler>,
    pub agent: Option<AgentToolsHandler>,
    pub experience: Option<ExperienceToolsHandler>,
    pub exploration: Option<ExplorationToolsHandler>,
    pub hypothesis: Option<HypothesisToolsHandler>,
    pub ingestor: Option<IngestorToolsHandler>,
    pub knowledge: Option<KnowledgeToolsHandler>,
    pub memory: Option<MemoryToolsHandler>,
    pub planner: Option<PlannerToolsHandler>,
    pub reflection: Option<ReflectionToolsHandler>,
    pub search: Option<SearchToolsHandler>,
    pub skills: Option<SkillsToolsHandler>,
    pub workflow: Option<WorkflowToolsHandler>,
}

impl Default for ToolHandlerCollection {
    fn default() -> Self {
        Self {
            acp: None,
            agent: None,
            experience: None,
            exploration: None,
            hypothesis: None,
            ingestor: None,
            knowledge: None,
            memory: None,
            planner: None,
            reflection: None,
            search: None,
            skills: None,
            workflow: None,
        }
    }
}

impl ToolHandlerCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize all handlers with graceful degradation
    /// 
    /// Returns a vector of any errors encountered during initialization.
    /// Handlers that fail to initialize are set to None and the system continues.
    pub fn initialize_all(
        context: Arc<crate::bridge::mcp::McpContext>,
        enforcer: Arc<crate::workflows::enforcement::WorkflowEnforcer>,
    ) -> (Self, Vec<HandlerInitError>) {
        let mut collection = Self::new();
        let mut errors = Vec::new();

        // Initialize each handler, capturing errors but continuing
        match AcpToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("ACP tools handler initialized with {} tools", handler.tool_names().len());
                collection.acp = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize ACP tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match AgentToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Agent tools handler initialized with {} tools", handler.tool_names().len());
                collection.agent = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize agent tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match ExperienceToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Experience tools handler initialized with {} tools", handler.tool_names().len());
                collection.experience = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize experience tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match ExplorationToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Exploration tools handler initialized with {} tools", handler.tool_names().len());
                collection.exploration = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize exploration tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match HypothesisToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Hypothesis tools handler initialized with {} tools", handler.tool_names().len());
                collection.hypothesis = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize hypothesis tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match IngestorToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Ingestor tools handler initialized with {} tools", handler.tool_names().len());
                collection.ingestor = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize ingestor tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match KnowledgeToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Knowledge tools handler initialized with {} tools", handler.tool_names().len());
                collection.knowledge = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize knowledge tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match MemoryToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Memory tools handler initialized with {} tools", handler.tool_names().len());
                collection.memory = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize memory tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match PlannerToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Planner tools handler initialized with {} tools", handler.tool_names().len());
                collection.planner = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize planner tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match ReflectionToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Reflection tools handler initialized with {} tools", handler.tool_names().len());
                collection.reflection = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize reflection tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match SearchToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Search tools handler initialized with {} tools", handler.tool_names().len());
                collection.search = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize search tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match SkillsToolsHandler::new(context.clone(), enforcer.clone()) {
            Ok(handler) => {
                tracing::info!("Skills tools handler initialized with {} tools", handler.tool_names().len());
                collection.skills = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize skills tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match WorkflowToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Workflow tools handler initialized with {} tools", handler.tool_names().len());
                collection.workflow = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize workflow tools handler: {}", e.message);
                errors.push(e);
            }
        }

        let total_tools = collection.count_tools();
        tracing::info!("Tool handlers initialization complete: {} total tools loaded, {} errors", 
            total_tools, errors.len());

        (collection, errors)
    }

    /// Count total number of tools across all handlers
    pub fn count_tools(&self) -> usize {
        let mut count = 0;
        if let Some(ref h) = self.acp { count += h.tool_names().len(); }
        if let Some(ref h) = self.agent { count += h.tool_names().len(); }
        if let Some(ref h) = self.experience { count += h.tool_names().len(); }
        if let Some(ref h) = self.exploration { count += h.tool_names().len(); }
        if let Some(ref h) = self.hypothesis { count += h.tool_names().len(); }
        if let Some(ref h) = self.ingestor { count += h.tool_names().len(); }
        if let Some(ref h) = self.knowledge { count += h.tool_names().len(); }
        if let Some(ref h) = self.memory { count += h.tool_names().len(); }
        if let Some(ref h) = self.planner { count += h.tool_names().len(); }
        if let Some(ref h) = self.reflection { count += h.tool_names().len(); }
        if let Some(ref h) = self.search { count += h.tool_names().len(); }
        if let Some(ref h) = self.skills { count += h.tool_names().len(); }
        if let Some(ref h) = self.workflow { count += h.tool_names().len(); }
        count
    }

    /// Check overall health of all handlers
    pub fn is_healthy(&self) -> bool {
        // At least one handler should be healthy
        self.acp.as_ref().map_or(false, |h| h.is_healthy())
            || self.agent.as_ref().map_or(false, |h| h.is_healthy())
            || self.experience.as_ref().map_or(false, |h| h.is_healthy())
            || self.exploration.as_ref().map_or(false, |h| h.is_healthy())
            || self.hypothesis.as_ref().map_or(false, |h| h.is_healthy())
            || self.ingestor.as_ref().map_or(false, |h| h.is_healthy())
            || self.knowledge.as_ref().map_or(false, |h| h.is_healthy())
            || self.memory.as_ref().map_or(false, |h| h.is_healthy())
            || self.planner.as_ref().map_or(false, |h| h.is_healthy())
            || self.reflection.as_ref().map_or(false, |h| h.is_healthy())
            || self.search.as_ref().map_or(false, |h| h.is_healthy())
            || self.skills.as_ref().map_or(false, |h| h.is_healthy())
            || self.workflow.as_ref().map_or(false, |h| h.is_healthy())
    }

    /// Get all tools from all handlers as MCP Tool definitions
    pub fn get_all_tools(&self) -> Vec<rmcp::model::Tool> {
        let mut tools = Vec::new();
        if let Some(ref h) = self.acp { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.agent { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.experience { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.exploration { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.hypothesis { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.ingestor { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.knowledge { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.memory { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.planner { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.reflection { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.search { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.skills { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.workflow { tools.extend(h.get_tools()); }
        tools
    }

    /// Get a single tool by name from any handler
    pub fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        if let Some(ref h) = self.acp { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.agent { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.experience { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.exploration { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.hypothesis { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.ingestor { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.knowledge { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.memory { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.planner { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.reflection { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.search { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.skills { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        if let Some(ref h) = self.workflow { 
            if let Some(tool) = h.get_tools().into_iter().find(|t| t.name == name) {
                return Some(tool);
            }
        }
        None
    }

    /// Call a tool by name with arguments
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<crate::bridge::tools::ToolOutput, HandlerError> {
        // Try each handler in order
        if let Some(ref h) = self.acp {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.agent {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.experience {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.exploration {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.hypothesis {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.ingestor {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.knowledge {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.memory {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.planner {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.reflection {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.search {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.skills {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        if let Some(ref h) = self.workflow {
            if h.tool_names().contains(&name.to_string()) {
                return h.execute_tool(name, args).await;
            }
        }
        Err(HandlerError::ToolNotFound(name.to_string()))
    }
}
