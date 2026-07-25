// src/workflows/enforcement.rs
//! Workflow enforcement layer - ensures agents follow mandatory workflow steps
//! 
//! This module provides enforcement for the required workflow:
//! 1. get_workflow - MUST be called first to retrieve workflow rules
//! 2. search_memory - MUST be called before any substantive action
//! 3. get_patterns - SHOULD be called for repetitive decisions
//! 4. Other tools - Only available after mandatory steps

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Tools that are exempt from workflow enforcement (always allowed)
const EXEMPT_TOOLS: &[&str] = &[
    "get_workflow",
    "list_tools",
    "get_tool",
];

/// Tools that count as "memory search" step
const MEMORY_SEARCH_TOOLS: &[&str] = &[
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

/// Workflow enforcement engine
pub struct WorkflowEnforcer {
    /// Per-session state tracking
    sessions: Arc<RwLock<std::collections::HashMap<String, SessionState>>>,
    /// Configuration
    enforcement_enabled: bool,
    session_timeout: Duration,
    require_memory_search: bool,
    require_patterns_review: bool,
}

impl WorkflowEnforcer {
    /// Create a new workflow enforcer
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            enforcement_enabled: true,
            session_timeout: Duration::from_secs(3600), // 1 hour default
            require_memory_search: true,
            require_patterns_review: false,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        enforcement_enabled: bool,
        session_timeout_secs: u64,
        require_memory_search: bool,
        require_patterns_review: bool,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            enforcement_enabled,
            session_timeout: Duration::from_secs(session_timeout_secs),
            require_memory_search,
            require_patterns_review,
        }
    }

    /// Get or create session state
    async fn get_session(&self, session_id: &str) -> SessionState {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            // Check if session expired
            if state.is_session_expired(self.session_timeout) {
                // Create fresh session
                let new_state = SessionState::new(session_id.to_string());
                *state = new_state.clone();
                return new_state;
            }
            state.last_activity = Instant::now();
            return state.clone();
        }
        
        let new_state = SessionState::new(session_id.to_string());
        sessions.insert(session_id.to_string(), new_state.clone());
        new_state
    }

    /// Record that get_workflow was called
    pub async fn record_workflow_retrieved(&self, session_id: &str, purpose: Option<String>) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.workflow_retrieved = true;
            state.workflow_purpose = purpose;
            state.last_activity = Instant::now();
        }
    }

    /// Record that memory search was called
    pub async fn record_memory_searched(&self, session_id: &str, query: Option<String>) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.memory_searched = true;
            state.last_memory_search = query;
            state.last_activity = Instant::now();
        }
    }

    /// Record that patterns were reviewed
    pub async fn record_patterns_reviewed(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.patterns_reviewed = true;
            state.last_activity = Instant::now();
        }
    }

    /// Check if a tool is exempt from enforcement
    pub fn is_exempt(tool_name: &str) -> bool {
        EXEMPT_TOOLS.iter().any(|&t| t == tool_name)
    }

    /// Check if a tool counts as memory search
    pub fn is_memory_search(tool_name: &str) -> bool {
        MEMORY_SEARCH_TOOLS.iter().any(|&t| t == tool_name)
    }

    /// Main enforcement check - returns error if tool should be blocked
    pub async fn check_enforcement(
        &self,
        session_id: &str,
        tool_name: &str,
    ) -> Result<(), WorkflowEnforcementError> {
        // If enforcement disabled, allow all
        if !self.enforcement_enabled {
            return Ok(());
        }

        // Exempt tools always allowed
        if Self::is_exempt(tool_name) {
            return Ok(());
        }

        let session = self.get_session(session_id).await;

        // Check if workflow was retrieved
        if !session.workflow_retrieved {
            return Err(WorkflowEnforcementError::workflow_not_retrieved());
        }

        // Check if memory was searched (for non-exempt, non-memory tools)
        if !Self::is_memory_search(tool_name) && self.require_memory_search && !session.memory_searched {
            return Err(WorkflowEnforcementError::memory_not_searched());
        }

        // All checks passed
        Ok(())
    }

    /// Record tool usage after successful execution
    pub async fn record_tool_execution(
        &self,
        session_id: &str,
        tool_name: &str,
        query: Option<String>,
    ) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.record_tool_use(tool_name);
            state.last_activity = Instant::now();

            // Record special milestones
            if tool_name == "get_workflow" {
                // Purpose should be updated via dedicated method
            } else if Self::is_memory_search(tool_name) {
                state.memory_searched = true;
                state.last_memory_search = query;
            } else if tool_name == "get_patterns" || tool_name == "analyze_patterns" {
                state.patterns_reviewed = true;
            }
        }
    }

    /// Get session state for debugging/admin purposes
    pub async fn get_session_state(&self, session_id: &str) -> Option<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Clear expired sessions
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();
        sessions.retain(|_, state| !state.is_session_expired(self.session_timeout));
        before - sessions.len()
    }

    /// Enable/disable enforcement at runtime
    pub fn set_enforcement_enabled(&mut self, enabled: bool) {
        self.enforcement_enabled = enabled;
    }

    /// Get current enforcement status
    pub fn is_enforcement_enabled(&self) -> bool {
        self.enforcement_enabled
    }

    /// Update session with workflow purpose
    pub async fn update_workflow_purpose(&self, session_id: &str, purpose: String) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.workflow_purpose = Some(purpose);
        }
    }
}

impl Default for WorkflowEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exempt_tools_always_allowed() {
        let enforcer = WorkflowEnforcer::new();
        let session_id = "test-session";
        
        // get_workflow should be allowed without any prior steps
        let result = enforcer.check_enforcement(session_id, "get_workflow").await;
        assert!(result.is_ok());

        // list_tools should be allowed
        let result = enforcer.check_enforcement(session_id, "list_tools").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_must_be_retrieved() {
        let enforcer = WorkflowEnforcer::new();
        let session_id = "test-session";
        
        // Without calling get_workflow, tools should be blocked
        let result = enforcer.check_enforcement(session_id, "store_memory").await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.error_code, "WORKFLOW_NOT_RETRIEVED");
        }
    }

    #[tokio::test]
    async fn test_memory_search_required() {
        let enforcer = WorkflowEnforcer::new();
        let session_id = "test-session";
        
        // Record workflow retrieval
        enforcer.record_workflow_retrieved(session_id, Some("general".to_string())).await;
        
        // Now memory search is required before other tools
        let result = enforcer.check_enforcement(session_id, "store_memory").await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.error_code, "MEMORY_NOT_SEARCHED");
        }
    }

    #[tokio::test]
    async fn test_memory_search_tools_allowed() {
        let enforcer = WorkflowEnforcer::new();
        let session_id = "test-session";
        
        // Record workflow retrieval
        enforcer.record_workflow_retrieved(session_id, Some("general".to_string())).await;
        
        // Memory search tools should be allowed
        let result = enforcer.check_enforcement(session_id, "search_memory").await;
        assert!(result.is_ok());

        let result = enforcer.check_enforcement(session_id, "list_memories").await;
        assert!(result.is_ok());

        let result = enforcer.check_enforcement(session_id, "get_patterns").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_full_workflow_completion() {
        let enforcer = WorkflowEnforcer::new();
        let session_id = "test-session";
        
        // Step 1: get_workflow
        enforcer.check_enforcement(session_id, "get_workflow").await.unwrap();
        enforcer.record_tool_execution(session_id, "get_workflow", None).await;
        
        // Step 2: search_memory
        enforcer.check_enforcement(session_id, "search_memory").await.unwrap();
        enforcer.record_tool_execution(session_id, "search_memory", Some("test query".to_string())).await;
        
        // Now other tools should be allowed
        let result = enforcer.check_enforcement(session_id, "store_memory").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_disabled_enforcement() {
        let mut enforcer = WorkflowEnforcer::with_config(
            false, // enforcement disabled
            3600,
            true,
            false
        );
        let session_id = "test-session";
        
        // Without calling get_workflow, all tools should be allowed
        let result = enforcer.check_enforcement(session_id, "store_memory").await;
        assert!(result.is_ok());
    }
}
