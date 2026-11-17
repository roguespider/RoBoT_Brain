//! Context lifecycle — Per Architecture §07 (Context Lifecycle).
//! Manages context lifecycle states and transitions.

/// Context lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextLifecycleState {
    /// Context is being assembled.
    Assembling,
    /// Context is ready for use.
    Ready,
    /// Context is deprecated.
    Deprecated,
    /// Context is archived.
    Archived,
}

impl Default for ContextLifecycleState {
    fn default() -> Self {
        Self::Assembling
    }
}

/// Context lifecycle manager.
pub struct ContextLifecycle {
    state: ContextLifecycleState,
}

impl ContextLifecycle {
    /// Create a new context lifecycle manager.
    pub fn new() -> Self {
        Self {
            state: ContextLifecycleState::Assembling,
        }
    }

    /// Transition to ready state.
    pub fn mark_ready(&mut self) {
        self.state = ContextLifecycleState::Ready;
    }

    /// Transition to deprecated state.
    pub fn mark_deprecated(&mut self) {
        self.state = ContextLifecycleState::Deprecated;
    }

    /// Transition to archived state.
    pub fn mark_archived(&mut self) {
        self.state = ContextLifecycleState::Archived;
    }

    /// Get current state.
    pub fn state(&self) -> ContextLifecycleState {
        self.state.clone()
    }

    /// Check if context is ready.
    pub fn is_ready(&self) -> bool {
        self.state == ContextLifecycleState::Ready
    }

    /// Finalize a context through the lifecycle.
    pub fn finalize(
        &self,
        context: crate::context_engine::coordinator::Context,
    ) -> crate::context_engine::coordinator::Context {
        crate::context_engine::coordinator::Context {
            complete: self.is_ready(),
            lifecycle_state: self.state.clone(),
            ..context
        }
    }
}
