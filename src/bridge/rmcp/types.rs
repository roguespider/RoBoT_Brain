

// src/bridge/rmcp/types.rs
// McpServerHandler struct definition


use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;

use crate::bridge::mcp::McpContext;
use crate::workflows::enforcement::{WorkflowEnforcer, WorkflowEnforcementError};
use crate::bridge::mcp::handlers::{ToolHandlerCollection, HandlerInitError};

/// MCP Server handler using the rmcp derive macros
/// 
/// Architecture:
/// - McpServerHandler loads MCP core first
/// - Then initializes all tool handlers independently
/// - Each tool handler can fail without affecting others
/// - Graceful degradation: if a handler fails, log warning but continue
#[derive(Clone)]
pub struct McpServerHandler {
    pub context: Arc<McpContext>,
    pub name: String,
    pub version: String,
    pub enforcer: Arc<WorkflowEnforcer>,
    pub session_counter: Arc<AtomicU64>,
    pub session_id: String,
    pub handlers: ToolHandlerCollection,
    pub handler_errors: Vec<HandlerInitError>,
}

impl McpServerHandler {
    pub fn new(context: Arc<McpContext>, name: String, version: String) -> Self {
        Self {
            context,
            name,
            version,
            enforcer: Arc::new(WorkflowEnforcer::new()),
            session_counter: Arc::new(AtomicU64::new(1)),
            session_id: "default".to_string(),
            handlers: ToolHandlerCollection::new(),
            handler_errors: Vec::new(),
        }
    }

    /// Initialize all tool handlers with graceful degradation
    /// 
    /// If any handler fails to initialize, it's logged but the system continues.
    /// This ensures that a single broken tool doesn't prevent the MCP server from starting.
    pub fn initialize_handlers(&mut self) {
        let (handlers, errors) = ToolHandlerCollection::initialize_all(
            self.context.clone(),
            self.enforcer.clone(),
        );
        self.handlers = handlers;
        self.handler_errors = errors;
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

    /// Check if a specific handler is available
    pub fn is_handler_available(&self, category: &str) -> bool {
        match category {
            "agent" => self.handlers.agent.is_some(),
            "experience" => self.handlers.experience.is_some(),
            "exploration" => self.handlers.exploration.is_some(),
            "hypothesis" => self.handlers.hypothesis.is_some(),
            "ingestor" => self.handlers.ingestor.is_some(),
            "knowledge" => self.handlers.knowledge.is_some(),
            "memory" => self.handlers.memory.is_some(),
            "planner" => self.handlers.planner.is_some(),
            "reflection" => self.handlers.reflection.is_some(),
            "search" => self.handlers.search.is_some(),
            "skills" => self.handlers.skills.is_some(),
            "workflow" => self.handlers.workflow.is_some(),
            _ => false,
        }
    }

    /// Get total count of available tools
    pub fn total_tools(&self) -> usize {
        self.handlers.count_tools()
    }

    /// Check if the server is healthy (at least one handler is available)
    pub fn is_healthy(&self) -> bool {
        self.handlers.is_healthy()
    }
}
