

// src/bridge/rmcp/types.rs
// McpServerHandler struct definition


use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;

use crate::bridge::mcp::McpContext;
use crate::workflows::enforcement::{WorkflowEnforcer, WorkflowEnforcementError};

// Import tool handler traits
#[cfg(feature = "tools-memory")]
use crate::bridge::rmcp::generated::memory_tools::MemoryToolsHandler;
#[cfg(feature = "tools-memory")]
use crate::bridge::rmcp::generated::tool_traits::MemoryToolsHandlerTrait;

#[cfg(feature = "tools-experience")]
use crate::bridge::rmcp::generated::experience_tools::ExperienceToolsHandler;
#[cfg(feature = "tools-experience")]
use crate::bridge::rmcp::generated::tool_traits::ExperienceToolsHandlerTrait;

#[cfg(feature = "tools-reflection")]
use crate::bridge::rmcp::generated::reflection_tools::ReflectionToolsHandler;
#[cfg(feature = "tools-reflection")]
use crate::bridge::rmcp::generated::tool_traits::ReflectionToolsHandlerTrait;

#[cfg(feature = "tools-search")]
use crate::bridge::rmcp::generated::search_tools::SearchToolsHandler;
#[cfg(feature = "tools-search")]
use crate::bridge::rmcp::generated::tool_traits::SearchToolsHandlerTrait;

#[cfg(feature = "tools-ingestor")]
use crate::bridge::rmcp::generated::ingestor_tools::IngestorToolsHandler;
#[cfg(feature = "tools-ingestor")]
use crate::bridge::rmcp::generated::tool_traits::IngestorToolsHandlerTrait;

#[cfg(feature = "tools-agent")]
use crate::bridge::rmcp::generated::agent_tools::AgentToolsHandler;
#[cfg(feature = "tools-agent")]
use crate::bridge::rmcp::generated::tool_traits::AgentToolsHandlerTrait;

#[cfg(feature = "tools-hypothesis")]
use crate::bridge::rmcp::generated::hypothesis_tools::HypothesisToolsHandler;
#[cfg(feature = "tools-hypothesis")]
use crate::bridge::rmcp::generated::tool_traits::HypothesisToolsHandlerTrait;

#[cfg(feature = "tools-knowledge")]
use crate::bridge::rmcp::generated::knowledge_tools::KnowledgeToolsHandler;
#[cfg(feature = "tools-knowledge")]
use crate::bridge::rmcp::generated::tool_traits::KnowledgeToolsHandlerTrait;

#[cfg(feature = "tools-planner")]
use crate::bridge::rmcp::generated::planner_tools::PlannerToolsHandler;
#[cfg(feature = "tools-planner")]
use crate::bridge::rmcp::generated::tool_traits::PlannerToolsHandlerTrait;

#[cfg(feature = "tools-workflow")]
use crate::bridge::rmcp::generated::workflow_tools::WorkflowToolsHandler;
#[cfg(feature = "tools-workflow")]
use crate::bridge::rmcp::generated::tool_traits::WorkflowToolsHandlerTrait;

#[cfg(feature = "tools-exploration")]
use crate::bridge::rmcp::generated::exploration_tools::ExplorationToolsHandler;
#[cfg(feature = "tools-exploration")]
use crate::bridge::rmcp::generated::tool_traits::ExplorationToolsHandlerTrait;

#[cfg(feature = "tools-skills")]
use crate::bridge::rmcp::generated::skills_tools::SkillsToolsHandler;
#[cfg(feature = "tools-skills")]
use crate::bridge::rmcp::generated::tool_traits::SkillsToolsHandlerTrait;

/// MCP Server handler using the rmcp derive macros
/// Aggregates all tool sub-handlers for isolated tool loading
#[derive(Clone)]
pub struct McpServerHandler {
    pub context: Arc<McpContext>,
    pub name: String,
    pub version: String,
    pub enforcer: Arc<WorkflowEnforcer>,
    pub session_counter: Arc<AtomicU64>,
    pub session_id: String,
    
    // Tool sub-handlers - each loads independently
    #[cfg(feature = "tools-memory")]
    pub memory_tools: Option<MemoryToolsHandler>,
    
    #[cfg(feature = "tools-experience")]
    pub experience_tools: Option<ExperienceToolsHandler>,
    
    #[cfg(feature = "tools-reflection")]
    pub reflection_tools: Option<ReflectionToolsHandler>,
    
    #[cfg(feature = "tools-search")]
    pub search_tools: Option<SearchToolsHandler>,
    
    #[cfg(feature = "tools-ingestor")]
    pub ingestor_tools: Option<IngestorToolsHandler>,
    
    #[cfg(feature = "tools-agent")]
    pub agent_tools: Option<AgentToolsHandler>,
    
    #[cfg(feature = "tools-hypothesis")]
    pub hypothesis_tools: Option<HypothesisToolsHandler>,
    
    #[cfg(feature = "tools-knowledge")]
    pub knowledge_tools: Option<KnowledgeToolsHandler>,
    
    #[cfg(feature = "tools-planner")]
    pub planner_tools: Option<PlannerToolsHandler>,
    
    #[cfg(feature = "tools-workflow")]
    pub workflow_tools: Option<WorkflowToolsHandler>,
    
    #[cfg(feature = "tools-exploration")]
    pub exploration_tools: Option<ExplorationToolsHandler>,
    
    #[cfg(feature = "tools-skills")]
    pub skills_tools: Option<SkillsToolsHandler>,
}

impl McpServerHandler {
    pub fn new(context: Arc<McpContext>, name: String, version: String) -> Self {
        Self {
            context: context.clone(),
            name,
            version,
            enforcer: Arc::new(WorkflowEnforcer::new()),
            session_counter: Arc::new(AtomicU64::new(1)),
            session_id: "default".to_string(),
            
            // Initialize tool handlers
            #[cfg(feature = "tools-memory")]
            memory_tools: Some(MemoryToolsHandler::new()),
            
            #[cfg(feature = "tools-experience")]
            experience_tools: Some(ExperienceToolsHandler::new()),
            
            #[cfg(feature = "tools-reflection")]
            reflection_tools: Some(ReflectionToolsHandler::new()),
            
            #[cfg(feature = "tools-search")]
            search_tools: Some(SearchToolsHandler::new()),
            
            #[cfg(feature = "tools-ingestor")]
            ingestor_tools: Some(IngestorToolsHandler::new()),
            
            #[cfg(feature = "tools-agent")]
            agent_tools: Some(AgentToolsHandler::new()),
            
            #[cfg(feature = "tools-hypothesis")]
            hypothesis_tools: Some(HypothesisToolsHandler::new()),
            
            #[cfg(feature = "tools-knowledge")]
            knowledge_tools: Some(KnowledgeToolsHandler::new()),
            
            #[cfg(feature = "tools-planner")]
            planner_tools: Some(PlannerToolsHandler::new()),
            
            #[cfg(feature = "tools-workflow")]
            workflow_tools: Some(WorkflowToolsHandler::new()),
            
            #[cfg(feature = "tools-exploration")]
            exploration_tools: Some(ExplorationToolsHandler::new()),
            
            #[cfg(feature = "tools-skills")]
            skills_tools: Some(SkillsToolsHandler::new()),
        }
    }

    pub fn new_session(&self) -> String {
        let id = self.session_counter.fetch_add(1, Ordering::SeqCst);
        format!("session-{}", id)
    }

    pub async fn check_workflow_enforcement(&self, tool_name: &str) -> Result<(), WorkflowEnforcementError> {
        self.enforcer.check_enforcement(&self.session_id, tool_name).await
    }

    pub async fn record_tool_execution(&self, tool_name: &str, query: Option<String>) {
        self.enforcer.record_tool_execution(&self.session_id, tool_name, query).await;
    }
}
