//! Context Lifecycle — Full lifecycle management for temporary cognitive environments
//! (Architecture Chapter 15 — Context Lifecycle).
//!
//! Per Architecture §15.1-15.11:
//! - Context is temporary, disposable, assembled on demand
//! - Lifecycle stages: Input Acquisition -> Intent Extraction -> Goal Activation ->
//!   Context Assembly -> Context Expansion -> Active Reasoning -> Continuous Refinement ->
//!   Reflection -> Context Compression -> Context Preservation -> Context Disposal
//! - Context does not own persistent memory or knowledge
//! - Wiring: context_engine/ -> context_lifecycle/ -> memory/ -> experience/ -> planner/

/// Lifecycle stages for context management (Chapter 15.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleStage {
    /// 1. Input Acquisition.
    InputAcquisition,
    /// 2. Intent Extraction.
    IntentExtraction,
    /// 3. Goal Activation.
    GoalActivation,
    /// 4. Context Assembly.
    ContextAssembly,
    /// 5. Context Expansion.
    ContextExpansion,
    /// 6. Active Reasoning.
    ActiveReasoning,
    /// 7. Continuous Refinement.
    ContinuousRefinement,
    /// 8. Reflection.
    Reflection,
    /// 9. Context Compression.
    ContextCompression,
    /// 10. Context Preservation.
    ContextPreservation,
    /// 11. Context Disposal.
    ContextDisposal,
}

impl LifecycleStage {
    /// Return stage label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::InputAcquisition => "InputAcquisition",
            Self::IntentExtraction => "IntentExtraction",
            Self::GoalActivation => "GoalActivation",
            Self::ContextAssembly => "ContextAssembly",
            Self::ContextExpansion => "ContextExpansion",
            Self::ActiveReasoning => "ActiveReasoning",
            Self::ContinuousRefinement => "ContinuousRefinement",
            Self::Reflection => "Reflection",
            Self::ContextCompression => "ContextCompression",
            Self::ContextPreservation => "ContextPreservation",
            Self::ContextDisposal => "ContextDisposal",
        }
    }
}

/// A context lifecycle tracking the full lifecycle of a temporary cognitive environment.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextLifecycle {
    /// Correlation ID linking this lifecycle to a conversation/session.
    pub correlation_id: String,
    /// Current lifecycle stage.
    pub stage: LifecycleStage,
    /// Whether the lifecycle is active.
    pub active: bool,
    /// Whether the lifecycle has completed.
    pub completed: bool,
    /// Whether the lifecycle has been disposed.
    pub disposed: bool,
    /// Timestamp when lifecycle started (epoch seconds).
    pub started_at: i64,
    /// Timestamp when lifecycle completed (epoch seconds).
    pub completed_at: Option<i64>,
}

impl ContextLifecycle {
    /// Create a new context lifecycle.
    pub fn new(correlation_id: &str) -> Self {
        Self {
            correlation_id: correlation_id.to_string(),
            stage: LifecycleStage::InputAcquisition,
            active: true,
            completed: false,
            disposed: false,
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }

    /// Advance to the next lifecycle stage.
    pub fn advance(&mut self) {
        if self.completed || self.disposed {
            return;
        }
        self.stage = match self.stage {
            LifecycleStage::InputAcquisition => LifecycleStage::IntentExtraction,
            LifecycleStage::IntentExtraction => LifecycleStage::GoalActivation,
            LifecycleStage::GoalActivation => LifecycleStage::ContextAssembly,
            LifecycleStage::ContextAssembly => LifecycleStage::ContextExpansion,
            LifecycleStage::ContextExpansion => LifecycleStage::ActiveReasoning,
            LifecycleStage::ActiveReasoning => LifecycleStage::ContinuousRefinement,
            LifecycleStage::ContinuousRefinement => LifecycleStage::Reflection,
            LifecycleStage::Reflection => LifecycleStage::ContextCompression,
            LifecycleStage::ContextCompression => LifecycleStage::ContextPreservation,
            LifecycleStage::ContextPreservation => LifecycleStage::ContextDisposal,
            LifecycleStage::ContextDisposal => LifecycleStage::ContextDisposal,
        };
        if self.stage == LifecycleStage::ContextDisposal {
            self.completed = true;
            self.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    /// Dispose of the lifecycle (final stage).
    pub fn dispose(&mut self) {
        self.stage = LifecycleStage::ContextDisposal;
        self.active = false;
        self.completed = true;
        self.disposed = true;
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// Check if lifecycle is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        self.completed || self.disposed
    }
}

impl Default for ContextLifecycle {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Active reference to lifecycle stages to prevent dead-code warnings.
pub fn reference_context_lifecycle() {
    let stages = vec![
        LifecycleStage::InputAcquisition,
        LifecycleStage::IntentExtraction,
        LifecycleStage::GoalActivation,
        LifecycleStage::ContextAssembly,
        LifecycleStage::ContextExpansion,
        LifecycleStage::ActiveReasoning,
        LifecycleStage::ContinuousRefinement,
        LifecycleStage::Reflection,
        LifecycleStage::ContextCompression,
        LifecycleStage::ContextPreservation,
        LifecycleStage::ContextDisposal,
    ];
    for stage in stages {
        tracing::debug!(stage = %stage.label(), "Context lifecycle stage referenced");
    }
}
