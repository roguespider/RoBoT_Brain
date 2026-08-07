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
}

/// Marker trait for handlers that need workflow enforcement
pub trait WorkflowEnforced {
    fn check_enforcement(&self, tool_name: &str) -> impl std::future::Future<Output = Result<(), String>> + Send;
    fn record_execution(&self, tool_name: &str, query: Option<String>) -> impl std::future::Future<Output = ()> + Send;
}

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
        self.agent.as_ref().map_or(false, |h| h.is_healthy())
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
}
