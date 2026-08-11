// src/planner/engine/mod.rs
//! Core planning engine for task decomposition and execution
//!
//! Per Architecture §2.8, §10:
//! Planning converts knowledge and goals into action.
//! Planning uses accumulated knowledge to make decisions.
//!
//! Per Architecture §5.7 Decision Flow:
//! Goal → Planning → Memory Retrieval → Knowledge Retrieval → Experience Retrieval → Confidence Evaluation → Action Selection → Execution → Outcome Recording

#[cfg(test)]
mod actions;
pub mod planner;
#[cfg(test)]
mod replanning;
pub mod types;

pub use planner::Planner;
pub use types::{Plan, PlanStatus};
