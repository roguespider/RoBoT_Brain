// src/workflows/enforcement.rs
//! Workflow enforcement layer - ensures agents follow mandatory workflow steps
//!
//! This module provides enforcement for the required workflow:
//! 1. get_workflow - MUST be called first to retrieve workflow rules
//! 2. search_memory - MUST be called before any substantive action
//! 3. get_patterns - SHOULD be called for repetitive decisions
//! 4. Other tools - Only available after mandatory steps

mod enforcer;

pub use enforcer::WorkflowEnforcer;

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Tools exempt from workflow enforcement — always allowed without prerequisites.
pub const EXEMPT_TOOLS: &[&str] = &["get_workflow", "list_tools", "get_tool"];

/// Tools that count as a memory search step (satisfy the memory-search requirement).
pub const MEMORY_SEARCH_TOOLS: &[&str] = &[
    "search_memory",
    "list_memories",
    "get_memory",
    "get_patterns",
    "query_knowledge",
];

/// Per-session workflow tracking state.
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub workflow_retrieved: bool,
    pub memory_searched: bool,
    pub patterns_reviewed: bool,
    pub workflow_purpose: Option<String>,
    pub last_memory_search: Option<String>,
    pub last_activity: Instant,
    pub created_at: Instant,
}

impl SessionState {
    /// Create a fresh session state for the given session id.
    pub fn new(session_id: String) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            workflow_retrieved: false,
            memory_searched: false,
            patterns_reviewed: false,
            workflow_purpose: None,
            last_memory_search: None,
            last_activity: now,
            created_at: now,
        }
    }

    /// Record that a tool was used during this session.
    pub fn record_tool_use(&mut self, tool_name: &str) {
        if tool_name == "get_workflow" {
            self.workflow_retrieved = true;
        } else if MEMORY_SEARCH_TOOLS.contains(&tool_name) {
            self.memory_searched = true;
        } else if tool_name == "get_patterns" || tool_name == "analyze_patterns" {
            self.patterns_reviewed = true;
        }
    }

    /// Has this session expired (no activity within the timeout)?
    pub fn is_session_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Summary view for admin/debug tooling.
    pub fn to_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.session_id,
            "workflow_retrieved": self.workflow_retrieved,
            "memory_searched": self.memory_searched,
            "patterns_reviewed": self.patterns_reviewed,
            "workflow_purpose": self.workflow_purpose,
            "last_memory_search": self.last_memory_search,
            "session_age_seconds": self.created_at.elapsed().as_secs(),
        })
    }
}

/// Error returned when a tool call is blocked by workflow enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEnforcementError {
    pub error_code: String,
    pub message: String,
    pub required_action: String,
    pub tools_blocked: Vec<String>,
}

impl WorkflowEnforcementError {
    /// Agent must call get_workflow first.
    pub fn workflow_not_retrieved() -> Self {
        Self {
            error_code: "WORKFLOW_NOT_RETRIEVED".to_string(),
            message: "Workflow rules have not been retrieved. Call get_workflow first.".to_string(),
            required_action: "get_workflow".to_string(),
            tools_blocked: Vec::new(),
        }
    }

    /// Agent must search memory before taking substantive action.
    pub fn memory_not_searched() -> Self {
        Self {
            error_code: "MEMORY_NOT_SEARCHED".to_string(),
            message: "Memory has not been searched. Call search_memory before using other tools."
                .to_string(),
            required_action: "search_memory".to_string(),
            tools_blocked: Vec::new(),
        }
    }

    /// Multiple tools blocked at once.
    pub fn tools_blocked(tools: Vec<String>) -> Self {
        Self {
            error_code: "TOOLS_BLOCKED".to_string(),
            message: format!("{} tool(s) blocked by workflow enforcement", tools.len()),
            required_action: "get_workflow".to_string(),
            tools_blocked: tools,
        }
    }
}
