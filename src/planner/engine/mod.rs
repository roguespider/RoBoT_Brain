// src/planner/engine/mod.rs
//! Core planning engine for task decomposition and execution
//!
//! Per Architecture §2.8, §10:
//! Planning converts knowledge and goals into action.
//! Planning uses accumulated knowledge to make decisions.
//!
//! Per Architecture §5.7 Decision Flow:
//! Goal → Planning → Memory Retrieval → Knowledge Retrieval → Experience Retrieval → Confidence Evaluation → Action Selection → Execution → Outcome Recording

mod actions;
pub mod planner;
mod replanning;
mod types;

pub use actions::{calculate_knowledge_confidence, score_action, select_best_scored};
pub use planner::Planner;
pub use replanning::{
    analyze_plan_failure, carry_forward_completed_steps, collect_completed_step_ids,
    create_replan, estimate_problem_complexity, reset_failed_steps,
};
pub use types::{
    ActionCandidate, ExperienceRef, KnowledgeRef, Plan, PlanFailureAnalysis, PlanStatus,
    PlanStep, PlannerPolicy, PlannerStats, ReplanReason, RiskLevel, StepStatus,
};
