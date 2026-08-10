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
pub mod personality_handler;
pub mod planner_handler;
pub mod reflection_handler;
pub mod search_handler;
pub mod skills_handler;
pub mod workflow_handler;
pub mod world_model_handler;

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
    /// Tool execution failed
    ExecutionFailed(String),
    /// Invalid parameters
    InvalidParams(String),
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::ToolNotFound(name) => write!(f, "Tool not found: {}", name),
            HandlerError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            HandlerError::InvalidParams(msg) => write!(f, "Invalid parameters: {}", msg),
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
    /// Each handler must implement this to handle its own tool execution.
    fn execute_tool(&self, name: &str, args: serde_json::Value) -> impl std::future::Future<Output = Result<crate::bridge::tools::ToolOutput, HandlerError>> + Send;
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
pub use personality_handler::PersonalityToolsHandler;
pub use planner_handler::PlannerToolsHandler;
pub use reflection_handler::ReflectionToolsHandler;
pub use search_handler::SearchToolsHandler;
pub use skills_handler::SkillsToolsHandler;
pub use workflow_handler::WorkflowToolsHandler;
pub use world_model_handler::WorldModelToolsHandler;

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
    pub personality: Option<PersonalityToolsHandler>,
    pub planner: Option<PlannerToolsHandler>,
    pub reflection: Option<ReflectionToolsHandler>,
    pub search: Option<SearchToolsHandler>,
    pub skills: Option<SkillsToolsHandler>,
    pub workflow: Option<WorkflowToolsHandler>,
    pub world_model: Option<WorldModelToolsHandler>,
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
            personality: None,
            planner: None,
            reflection: None,
            search: None,
            skills: None,
            workflow: None,
            world_model: None,
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

        match AgentToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Agent tools handler initialized with {} tools", handler.tool_names().len());
                collection.agent = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize agent tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match ExperienceToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Experience tools handler initialized with {} tools", handler.tool_names().len());
                collection.experience = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize experience tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match ExplorationToolsHandler::new() {
            Ok(handler) => {
                tracing::info!("Exploration tools handler initialized with {} tools", handler.tool_names().len());
                collection.exploration = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize exploration tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match HypothesisToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Hypothesis tools handler initialized with {} tools", handler.tool_names().len());
                collection.hypothesis = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize hypothesis tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match IngestorToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Ingestor tools handler initialized with {} tools", handler.tool_names().len());
                collection.ingestor = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize ingestor tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match KnowledgeToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Knowledge tools handler initialized with {} tools", handler.tool_names().len());
                collection.knowledge = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize knowledge tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match MemoryToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Memory tools handler initialized with {} tools", handler.tool_names().len());
                collection.memory = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize memory tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match PersonalityToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Personality tools handler initialized with {} tools", handler.tool_names().len());
                collection.personality = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize personality tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match PlannerToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Planner tools handler initialized with {} tools", handler.tool_names().len());
                collection.planner = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize planner tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match ReflectionToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Reflection tools handler initialized with {} tools", handler.tool_names().len());
                collection.reflection = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize reflection tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match SearchToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("Search tools handler initialized with {} tools", handler.tool_names().len());
                collection.search = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize search tools handler: {}", e.message);
                errors.push(e);
            }
        }

        match SkillsToolsHandler::new(context.clone()) {
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

        match WorldModelToolsHandler::new(context.clone()) {
            Ok(handler) => {
                tracing::info!("World-model tools handler initialized with {} tools", handler.tool_names().len());
                collection.world_model = Some(handler);
            }
            Err(e) => {
                tracing::warn!("Failed to initialize world-model tools handler: {}", e.message);
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
        if let Some(ref h) = self.personality { count += h.tool_names().len(); }
        if let Some(ref h) = self.planner { count += h.tool_names().len(); }
        if let Some(ref h) = self.reflection { count += h.tool_names().len(); }
        if let Some(ref h) = self.search { count += h.tool_names().len(); }
        if let Some(ref h) = self.skills { count += h.tool_names().len(); }
        if let Some(ref h) = self.workflow { count += h.tool_names().len(); }
        if let Some(ref h) = self.world_model { count += h.tool_names().len(); }
        count
    }

    /// Check overall health of all handlers
    pub fn is_healthy(&self) -> bool {
        // At least one handler should be healthy
        self.acp.as_ref().is_some_and(|h| h.is_healthy())
            || self.agent.as_ref().is_some_and(|h| h.is_healthy())
            || self.experience.as_ref().is_some_and(|h| h.is_healthy())
            || self.exploration.as_ref().is_some_and(|h| h.is_healthy())
            || self.hypothesis.as_ref().is_some_and(|h| h.is_healthy())
            || self.ingestor.as_ref().is_some_and(|h| h.is_healthy())
            || self.knowledge.as_ref().is_some_and(|h| h.is_healthy())
            || self.memory.as_ref().is_some_and(|h| h.is_healthy())
            || self.personality.as_ref().is_some_and(|h| h.is_healthy())
            || self.planner.as_ref().is_some_and(|h| h.is_healthy())
            || self.reflection.as_ref().is_some_and(|h| h.is_healthy())
            || self.search.as_ref().is_some_and(|h| h.is_healthy())
            || self.skills.as_ref().is_some_and(|h| h.is_healthy())
            || self.workflow.as_ref().is_some_and(|h| h.is_healthy())
            || self.world_model.as_ref().is_some_and(|h| h.is_healthy())
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
        if let Some(ref h) = self.personality { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.planner { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.reflection { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.search { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.skills { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.workflow { tools.extend(h.get_tools()); }
        if let Some(ref h) = self.world_model { tools.extend(h.get_tools()); }
        tools
    }

    /// Get a single tool by name from any handler
    pub fn get_tool(&self, name: &str) -> Option<rmcp::model::Tool> {
        if let Some(tool) = self.acp.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.agent.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.experience.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.exploration.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.hypothesis.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.ingestor.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.knowledge.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.memory.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.personality.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.planner.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.reflection.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.search.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.skills.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.workflow.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        if let Some(tool) = self.world_model.as_ref().and_then(|h| h.get_tools().into_iter().find(|t| t.name == name)) {
            return Some(tool);
        }
        None
    }

    /// Call a tool by name with arguments
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<crate::bridge::tools::ToolOutput, HandlerError> {
        // Try each handler in order. Each handler's category() identifies
        // which subsystem processed the tool (useful for debugging).
        macro_rules! try_handler {
            ($handler:expr) => {
                if let Some(ref h) = $handler {
                    if h.tool_names().contains(&name.to_string()) {
                        tracing::debug!(
                            tool = name,
                            category = h.category(),
                            "Dispatching tool to handler"
                        );
                        return h.execute_tool(name, args).await;
                    }
                }
            };
        }

        try_handler!(self.acp);
        try_handler!(self.agent);
        try_handler!(self.experience);
        try_handler!(self.exploration);
        try_handler!(self.hypothesis);
        try_handler!(self.ingestor);
        try_handler!(self.knowledge);
        try_handler!(self.memory);
        try_handler!(self.personality);
        try_handler!(self.planner);
        try_handler!(self.reflection);
        try_handler!(self.search);
        try_handler!(self.skills);
        try_handler!(self.workflow);
        try_handler!(self.world_model);

        tracing::warn!(tool = name, "Tool not found in any handler category");
        Err(HandlerError::ToolNotFound(name.to_string()))
    }
}
