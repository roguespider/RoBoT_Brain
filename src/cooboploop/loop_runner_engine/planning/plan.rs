//! Planning logic for the CoObOpLoop.

use crate::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};

/// Generate a plan for the selected objective.
pub fn plan(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::Plan;
    // TODO: Implement plan generation logic
    tracing::debug!("plan: generating plan for selected objective");
    runner.current_plan = Some(crate::planner::engine::types::Plan::default());
}
