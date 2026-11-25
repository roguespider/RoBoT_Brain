//! Execution Lifecycle — State machine for execution (Architecture Chapter 12.6).
//!
//! Wiring: `execution/` -> `agent/loop_runner.rs` -> `database/`

/// Execution lifecycle states.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    /// Request received.
    Received,
    /// Request validated.
    Validated,
    /// Execution state initialized.
    Initialized,
    /// Resources allocated.
    ResourcesAllocated,
    /// Actions scheduled.
    Scheduled,
    /// Actions dispatched.
    Dispatched,
    /// Progress being monitored.
    Monitoring,
    /// Execution completed successfully.
    Completed,
    /// Execution failed.
    Failed,
    /// Execution cancelled.
    Cancelled,
    /// Results captured.
    ResultsCaptured,
}

/// The execution lifecycle state machine.
#[derive(Debug, Clone)]
pub struct ExecutionLifecycle {
    /// Current state.
    pub state: LifecycleState,
    /// Execution request ID.
    pub execution_id: String,
    /// Timestamp of last state change.
    pub last_updated: i64,
    /// State transition history.
    pub history: Vec<(LifecycleState, i64)>,
}

impl ExecutionLifecycle {
    /// Create a new lifecycle for an execution.
    pub fn new(execution_id: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            state: LifecycleState::Received,
            execution_id: execution_id.to_string(),
            last_updated: now,
            history: vec![(LifecycleState::Received, now)],
        }
    }

    /// Advance to the next state.
    pub fn advance(&mut self, new_state: LifecycleState) {
        self.history.push((self.state.clone(), self.last_updated));
        self.state = new_state;
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Get current state.
    pub fn current_state(&self) -> &LifecycleState {
        &self.state
    }

    /// Check if execution is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            LifecycleState::Completed | LifecycleState::Failed | LifecycleState::Cancelled
        )
    }
}

