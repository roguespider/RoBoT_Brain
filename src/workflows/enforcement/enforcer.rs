


// src/workflows/enforcement/enforcer.rs
//! WorkflowEnforcer implementation
//! 
//! Enforcement is ALWAYS enabled - agents MUST follow workflows.
//! There is no option to disable enforcement.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::workflows::enforcement::{SessionState, WorkflowEnforcementError, EXEMPT_TOOLS, MEMORY_SEARCH_TOOLS};

/// Workflow enforcement engine - ALWAYS ENABLED
pub struct WorkflowEnforcer {
    /// Per-session state tracking
    sessions: Arc<RwLock<std::collections::HashMap<String, SessionState>>>,
    /// Session timeout
    session_timeout: Duration,
}

impl WorkflowEnforcer {
    /// Create a new workflow enforcer (enforcement is always enabled)
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            session_timeout: Duration::from_secs(3600), // 1 hour default
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

    /// Check enforcement for multiple tools at once, returning a combined
    /// error listing all blocked tools if any fail.
    pub async fn check_multiple_tools(
        &self,
        session_id: &str,
        tool_names: &[String],
    ) -> Result<(), WorkflowEnforcementError> {
        let mut blocked: Vec<String> = Vec::new();
        for tool_name in tool_names {
            if Self::is_exempt(tool_name) {
                continue;
            }
            let session = self.get_session(session_id).await;
            if !session.workflow_retrieved || (!Self::is_memory_search(tool_name) && !session.memory_searched) {
                blocked.push(tool_name.clone());
            }
        }
        if blocked.is_empty() {
            Ok(())
        } else {
            Err(WorkflowEnforcementError::tools_blocked(blocked))
        }
    }

    /// Check if a tool is exempt from enforcement
    pub fn is_exempt(tool_name: &str) -> bool {
        EXEMPT_TOOLS.contains(&tool_name)
    }

    /// Check if a tool counts as memory search
    pub fn is_memory_search(tool_name: &str) -> bool {
        MEMORY_SEARCH_TOOLS.contains(&tool_name)
    }

    /// Main enforcement check - ALWAYS ENFORCED
    /// Returns error if agent hasn't followed required workflow steps
    pub async fn check_enforcement(
        &self,
        session_id: &str,
        tool_name: &str,
    ) -> Result<(), WorkflowEnforcementError> {
        // Exempt tools always allowed (get_workflow, list_tools, get_tool)
        if Self::is_exempt(tool_name) {
            if tool_name == "get_workflow" {
                self.record_workflow_retrieved(session_id, None).await;
            }
            return Ok(());
        }

        let session = self.get_session(session_id).await;

        // Agent MUST have called get_workflow first
        if !session.workflow_retrieved {
            return Err(WorkflowEnforcementError::workflow_not_retrieved());
        }

        // For non-memory tools, agent MUST have searched memory first
        if !Self::is_memory_search(tool_name) && !session.memory_searched {
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
