

// src/workflows/enforcement/enforcer.rs
//! WorkflowEnforcer implementation

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::workflows::enforcement::{SessionState, WorkflowEnforcementError, EXEMPT_TOOLS, MEMORY_SEARCH_TOOLS};

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
        } else {
            // Create new session with workflow_retrieved set to true
            let mut new_state = SessionState::new(session_id.to_string());
            new_state.workflow_retrieved = true;
            new_state.workflow_purpose = purpose;
            sessions.insert(session_id.to_string(), new_state);
        }
    }

    /// Record that memory search was called
    pub async fn record_memory_searched(&self, session_id: &str, query: Option<String>) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.memory_searched = true;
            state.last_memory_search = query;
            state.last_activity = Instant::now();
        } else {
            // Create new session with memory_searched set to true
            let mut new_state = SessionState::new(session_id.to_string());
            new_state.memory_searched = true;
            new_state.last_memory_search = query;
            sessions.insert(session_id.to_string(), new_state);
        }
    }

    /// Record that patterns were reviewed
    pub async fn record_patterns_reviewed(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(state) = sessions.get_mut(session_id) {
            state.patterns_reviewed = true;
            state.last_activity = Instant::now();
        } else {
            // Create new session with patterns_reviewed set to true
            let mut new_state = SessionState::new(session_id.to_string());
            new_state.patterns_reviewed = true;
            sessions.insert(session_id.to_string(), new_state);
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
            // For get_workflow, mark workflow as retrieved
            if tool_name == "get_workflow" {
                self.record_workflow_retrieved(session_id, None).await;
            }
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
