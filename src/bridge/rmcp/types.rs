#![allow(dead_code)]

// src/bridge/rmcp/types.rs
// McpServerHandler struct definition

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;

use crate::bridge::mcp::McpContext;
use crate::workflows::enforcement::{WorkflowEnforcer, WorkflowEnforcementError};

/// MCP Server handler using the rmcp derive macros
#[derive(Clone)]
pub struct McpServerHandler {
    pub context: Arc<McpContext>,
    pub name: String,
    pub version: String,
    pub enforcer: Arc<WorkflowEnforcer>,
    pub session_counter: Arc<AtomicU64>,
    pub session_id: String,
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

    pub async fn record_workflow_retrieved(&self, purpose: String) {
        self.enforcer.record_workflow_retrieved(&self.session_id, Some(purpose)).await;
    }
}
