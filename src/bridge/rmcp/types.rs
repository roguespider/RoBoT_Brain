

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
            context: context.clone(),
            name,
            version,
            enforcer: context.enforcer.clone(),
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
        );
        self.handlers = handlers;
        self.handler_errors = errors;

        // Log which handler categories are available for diagnostics.
        for cat in [
            "acp", "agent", "experience", "exploration", "hypothesis",
            "ingestor", "knowledge", "memory", "personality", "planner", "reflection",
            "search", "skills", "workflow", "world_model",
        ] {
            if self.is_handler_available(cat) {
                tracing::info!(category = cat, "Handler category available");
            } else {
                tracing::warn!(category = cat, "Handler category missing");
            }
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

    /// Record explicit memory search milestone (Architecture §22 workflow gate)
    pub async fn record_memory_searched(&self, query: Option<String>) {
        self.enforcer.record_memory_searched(&self.session_id, query).await;
    }

    /// Record explicit patterns-reviewed milestone
    pub async fn record_patterns_reviewed(&self) {
        self.enforcer.record_patterns_reviewed(&self.session_id).await;
    }

    /// Get the current session's enforcement state for debugging/admin
    pub async fn get_session_state(&self) -> Option<crate::workflows::enforcement::SessionState> {
        self.enforcer.get_session_state(&self.session_id).await
    }

    /// Clean up expired enforcement sessions and return how many were removed
    pub async fn cleanup_expired_sessions(&self) -> usize {
        self.enforcer.cleanup_expired_sessions().await
    }

    /// Update the workflow purpose for the current session
    pub async fn update_workflow_purpose(&self, purpose: String) {
        self.enforcer.update_workflow_purpose(&self.session_id, purpose).await;
    }

    /// Emit an `ExperienceRecorded` event for a tool execution outcome so the
    /// §4.04 learning spine advances automatically (Architecture §2.04:
    /// "Everything Important Becomes an Experience", TASK-V2-05).
    ///
    /// This publishes the event on the experience bus without persisting every
    /// tool call — the EventSubscriber (wired in P0) drives reflection /
    /// hypothesis / knowledge promotion from the event alone. Mutating tools
    /// and failures are also recorded to durable storage so they are
    /// retrievable by future loops.
    pub async fn emit_tool_experience(
        &self,
        tool_name: &str,
        success: bool,
        arguments: &serde_json::Value,
    ) {
        use crate::experience::types::{
            Experience, ExperienceContext, ExperienceOutcome, ExperienceType,
        };
        use crate::experience::types::context::ToolContext;
        use std::collections::HashMap;

        let outcome = if success {
            ExperienceOutcome::success()
        } else {
            ExperienceOutcome::failure("tool execution returned an error")
        };

        let mut experience = Experience::new(
            format!("Tool execution: {}", tool_name),
            format!("tool={} success={} args={}", tool_name, success, arguments),
            ExperienceType::ToolExecution,
            Vec::new(),
        );
        experience.context = ExperienceContext {
            tool: Some(ToolContext {
                name: tool_name.to_string(),
                version: None,
                arguments: HashMap::new(),
            }),
            session_id: Some(self.session_id.clone()),
            ..ExperienceContext::default()
        };
        experience.outcome = outcome;

        // process() scores the experience and publishes ExperienceRecorded
        // once (P0 V2-02). This drives reflection → hypothesis → knowledge
        // without the caller having to manually record.
        let processed = self.context.coordinator.process(experience);

        // Persist notable outcomes (failures and mutations) so they are
        // retrievable; read-only successes are kept as ephemeral events to
        // avoid flooding durable storage.
        let notable = !success
            || matches!(
                crate::agent::safety_gate::action_risk(tool_name),
                crate::agent::safety_gate::ActionRisk::Mutate
            );
        if notable {
            if let Ok(conn) = self.context.database.connection() {
                let memory =
                    crate::database::models::MemoryCard::from_experience(&processed);
                if let Err(e) = crate::database::queries::insert_memory(&conn, &memory) {
                    tracing::warn!(
                        "Failed to persist tool experience for {}: {}",
                        tool_name,
                        e
                    );
                }
            }
        }
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
            "personality" => self.handlers.personality.is_some(),
            "planner" => self.handlers.planner.is_some(),
            "reflection" => self.handlers.reflection.is_some(),
            "search" => self.handlers.search.is_some(),
            "skills" => self.handlers.skills.is_some(),
            "workflow" => self.handlers.workflow.is_some(),
            "world_model" => self.handlers.world_model.is_some(),
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
