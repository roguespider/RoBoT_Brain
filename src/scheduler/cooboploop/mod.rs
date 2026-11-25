//! CoObOpLoop — Continuous Objective & Opportunity Loop
//!
//! Architecture: CoObOpLoop.md, research_engine.md, RoBoT Architecture v0.0.2.md
//!
//! This module provides the continuous cognitive loop that enables RoBoT to
//! evaluate its environment, internal state, and available opportunities
//! without requiring external input after tasks complete.
//!
//! Per CoObOpLoop.md §1: The system SHALL NOT require a human to continuously
//! provide the next action. Completion of a task SHALL transition RoBoT into
//! an evaluation state rather than terminating cognitive activity.

// Sub-module declarations — one file per responsibility
pub mod capability;
pub mod evaluation;
pub mod event_tracer;
pub mod hardware;
pub mod human;
pub mod idle;
pub mod inspection;
pub mod learning_feedback;
pub mod learning_pipeline;
pub mod llm_provider;
pub mod loop_runner_engine;
pub mod opportunity;
pub mod post_task;
pub mod queue;
pub mod research;
pub mod self_improvement;
pub mod sources;
pub mod strategic;

// Re-export key types for backward compatibility
pub use capability::{CapabilityId, CapabilityRegistry};
pub use evaluation::{GoalEvaluator, PriorityPolicyRegistry, PriorityPolicyTrait};
pub use human::HumanActionHandler;
pub use idle::IdleState;
pub use inspection::Inspector;
pub use llm_provider::{DefaultRuntimeAdapters, DefaultToolRegistry};
pub use loop_runner_engine::{CyclePhase, LoopRunner};
pub use opportunity::OpportunityIntake;
pub use post_task::PostTaskEvaluation;
pub use queue::{AgentGoal, GoalStatus, ObjectiveQueue};
pub use research::ResearchManager;
pub use self_improvement::SelfImprovementPipeline;
pub use sources::{HumanOrigin, ObjectiveSource};
pub use strategic::{StrategicObjective, StrategicObjectiveRegistry};

/// Initialize the CoObOpLoop subsystem.
///
/// Called once during application startup (e.g. from `CooboploopToolsHandler::new`).
/// Seeds default capabilities and registers any built-in objective sources.
pub fn init(database_path: &str) -> CapabilityRegistry {
    tracing::debug!("CoObOpLoop: initializing");
    let mut registry =
        CapabilityRegistry::open(database_path).unwrap_or_else(|_| CapabilityRegistry::new());
    capability::seed_default_capabilities(&mut registry);
    tracing::debug!("CoObOpLoop: initialization complete");
    registry
}
