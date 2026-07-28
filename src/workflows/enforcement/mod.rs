
#![allow(dead_code)]

// src/workflows/enforcement.rs
//! Workflow enforcement layer - ensures agents follow mandatory workflow steps
//! 
//! This module provides enforcement for the required workflow:
//! 1. get_workflow - MUST be called first to retrieve workflow rules
//! 2. search_memory - MUST be called before any substantive action
//! 3. get_patterns - SHOULD be called for repetitive decisions
//! 4. Other tools - Only available after mandatory steps



mod enforcer;
#[cfg(test)]
mod tests;

pub use enforcer::WorkflowEnforcer;

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Tools that are exempt from workflow enforcement (always allowed)
pub const EXEMPT_TOOLS: &[&str] = &[
    "get_workflow",
    "list_tools",
    "get_tool",
];

/// Tools that count as "memory search" step
pub const MEMORY_SEARCH_TOOLS: &[&str] = &[
    "search_memory",
    "list_memories",
    "get_memory",
    "get_patterns",
    "get_insights",
    "global_search",
];

/// Workflow enforcement error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEnforcementError {
    pub error_code: String,
    pub message: String,
    pub required_action: String,
    pub tools_blocked: Vec<String>,
}

impl WorkflowEnforcementError {
    pub fn workflow_not_retrieved() -> Self {
        Self {
            error_code: "WORKFLOW_NOT_RETRIEVED".to_string(),
            message: "You MUST call 'get_workflow' first before using any other tool.".to_string(),
            required_action: "Call get_workflow with a purpose (e.g., 'general', 'file_ingestion', 'memory_search')".to_string(),
            tools_blocked: vec![],
        }
    }

    pub fn memory_not_searched() -> Self {
        Self {
            error_code: "MEMORY_NOT_SEARCHED".to_string(),
            message: "You MUST call 'search_memory' before taking any substantive action.".to_string(),
            required_action: "Call search_memory with a relevant query about your task".to_string(),
            tools_blocked: vec![],
        }
    }

    pub fn tools_blocked(blocked: Vec<String>) -> Self {
        Self {
            error_code: "TOOLS_BLOCKED".to_string(),
            message: "These tools require completing mandatory workflow steps first.".to_string(),
            required_action: "Complete mandatory workflow steps (get_workflow, search_memory)".to_string(),
            tools_blocked: blocked,
        }
    }
}

/// Session state tracking workflow compliance
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session_id: String,
    pub workflow_retrieved: bool,
    pub workflow_purpose: Option<String>,
    pub memory_searched: bool,
    pub last_memory_search: Option<String>,
    pub patterns_reviewed: bool,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub tools_used: Vec<String>,
}

impl SessionState {
    pub fn new(session_id: String) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            workflow_retrieved: false,
            workflow_purpose: None,
            memory_searched: false,
            last_memory_search: None,
            patterns_reviewed: false,
            created_at: now,
            last_activity: now,
            tools_used: Vec::new(),
        }
    }

    pub fn record_tool_use(&mut self, tool_name: &str) {
        self.last_activity = Instant::now();
        if !self.tools_used.contains(&tool_name.to_string()) {
            self.tools_used.push(tool_name.to_string());
        }
    }

    pub fn is_session_expired(&self, max_age: Duration) -> bool {
        self.last_activity.elapsed() > max_age
    }
}
