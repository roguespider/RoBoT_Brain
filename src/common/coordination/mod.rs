//! Cognitive coordination - Per Architecture §16

use crate::conversation::communication::events::{EventBus, InternalEvent};

/// Errors that can occur during coordination operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinationError {
    /// The decision has no chosen action to route.
    NoAction,
    /// The decision's chosen action is empty.
    EmptyAction,
    /// Unknown error during routing.
    Unknown(String),
}

impl std::fmt::Display for CoordinationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoordinationError::NoAction => write!(f, "Decision has no chosen action"),
            CoordinationError::EmptyAction => write!(f, "Chosen action is empty"),
            CoordinationError::Unknown(msg) => write!(f, "Unknown coordination error: {}", msg),
        }
    }
}

impl std::error::Error for CoordinationError {}

/// Orchestrator for subsystem coordination.
#[derive(Debug, Default)]
pub struct Orchestrator {
    pub event_bus: EventBus,
}

/// Dispatch an event through the orchestrator.
pub fn dispatch(orch: &Orchestrator, event: InternalEvent) -> Result<Vec<InternalEvent>, String> {
    tracing::debug!(
        subscriber_count = orch.event_bus.subscribers.len(),
        kind = %event.kind,
        source = %event.source,
        "Dispatching event through orchestrator"
    );
    Ok(vec![event])
}

/// Route a decision to an execution step.
///
/// Per Architecture §16 — decision routing validates the decision has a
/// non-empty action before producing an ExecutionStep.
pub fn route_decision(
    orch: &Orchestrator,
    decision: &crate::common::data_contracts::decision::Decision,
) -> Result<crate::execution::ExecutionStep, CoordinationError> {
    if decision.selected_plan.is_empty() {
        return Err(CoordinationError::EmptyAction);
    }
    tracing::debug!(
        subscriber_count = orch.event_bus.subscribers.len(),
        selected_plan = %decision.selected_plan,
        "Routing decision to execution step"
    );
    Ok(crate::execution::ExecutionStep::new(
        &decision.selected_plan,
        &decision.selected_plan,
    ))
}

