//! Plan-to-execution conversion for the planning engine

use super::Planner;

use crate::planning::engine::types::Plan;

impl Planner {
    /// Convert a plan to an execution request.
    /// Integration: planner -> execution (Architecture §12.5)
    pub fn plan_to_execution_request(&self, plan: &Plan) -> crate::execution::ExecutionRequest {
        let actions: Vec<String> = plan.steps.iter().map(|s| s.action.clone()).collect();
        crate::execution::execution_request_from_plan(&plan.id, &plan.goal, actions)
    }
}
