//! Agent module — re-exports the CoObOpLoop agent loop subsystem.
//!
//! Architecture v0.0.2 requires src/agent/ as the agent loop entry point.
//! The implementation lives in scheduler::cooboploop; this module provides
//! a canonical re-export path per the architecture chapter layout.
//!
//! The `decision` submodule implements the 9-tier confidence cascade
//! (research_engine.md §2) that gates research dispatch.

pub mod decision;

pub use crate::scheduler::cooboploop::{
    AgentGoal, CapabilityId, CapabilityRegistry, CyclePhase, GoalEvaluator, GoalStatus,
    HumanActionHandler, IdleState, Inspector, LoopRunner, ObjectiveQueue, ObjectiveSource,
    OpportunityIntake, PostTaskEvaluation, PriorityPolicyRegistry, PriorityPolicyTrait,
    ResearchManager, SelfImprovementPipeline, StrategicObjective, StrategicObjectiveRegistry,
};
