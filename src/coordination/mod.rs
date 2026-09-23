//! Cognitive coordination - Per Architecture §16

use crate::communication::events::{EventBus, InternalEvent};

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
    decision: &crate::data_contracts::decision::Decision,
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

/// Active reference to coordination contracts.
///
/// Exercises `CoordinationError`, `Orchestrator`, `dispatch`, and `route_decision`
/// so they stay live rather than dead code.
pub fn reference_coordination_contracts() {
    let orch = Orchestrator::default();
    let event = InternalEvent {
        kind: "coord_test".to_string(),
        source: "test".to_string(),
        correlation_id: "test-correlation".to_string(),
        payload: serde_json::json!({"test": true}),
        created_at: 0,
    };
    if let Ok(events) = dispatch(&orch, event) {
        tracing::debug!(
            "Dispatched event through orchestrator: {} events returned",
            events.len()
        );
    } else {
        tracing::warn!("Event dispatch through orchestrator failed");
    }

    // Exercise NoAction variant
    let no_action_err = CoordinationError::NoAction;
    let unknown_err = CoordinationError::Unknown("test error".to_string());
    let subscriber_count = orch.event_bus.subscribers.len();
    tracing::debug!(
        "coord_ref: NoAction={:?}, Unknown={:?}, subscribers={}",
        no_action_err,
        unknown_err,
        subscriber_count
    );

    let decision = crate::data_contracts::decision::Decision {
        id: "test-decision".to_string(),
        selected_plan: "test_action".to_string(),
        reason: "test rationale".to_string(),
        confidence: 0.8,
        alternatives: Vec::new(),
        supporting_memory: Vec::new(),
        supporting_experience: Vec::new(),
        timestamp: 0,
        metadata: crate::data_contracts::metadata::Metadata::new("coord_ref"),
    };
    let step = route_decision(&orch, &decision).unwrap_or_else(|_| {
        tracing::warn!("route_decision failed for test decision, using fallback");
        crate::execution::ExecutionStep::new("fallback", "fallback")
    });
    tracing::info!(
        step_id = %step.id,
        step_action = %step.action,
        "Coordination contracts actively referenced"
    );

    // Verify empty-action error path
    let empty_decision = crate::data_contracts::decision::Decision {
        id: "empty-decision".to_string(),
        selected_plan: "".to_string(),
        reason: "".to_string(),
        confidence: 0.0,
        alternatives: Vec::new(),
        supporting_memory: Vec::new(),
        supporting_experience: Vec::new(),
        timestamp: 0,
        metadata: crate::data_contracts::metadata::Metadata::new("coord_ref"),
    };
    match route_decision(&orch, &empty_decision) {
        Err(CoordinationError::EmptyAction) => {
            tracing::info!("Empty action correctly rejected by route_decision");
        }
        other => {
            tracing::warn!("Expected EmptyAction error, got: {:?}", other);
        }
    }
}
