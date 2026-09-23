use serde_json;
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::cooboploop::inputs::*;
use crate::cooboploop::evaluation::{GoalEvaluator, PriorityPolicyRegistry};
use crate::cooboploop::queue::{AgentGoal, ObjectiveQueue};

/// Execute cooboploop_run_source_discovery
pub async fn execute_cooboploop_run_source_discovery(
    input: CooboploopRunSourceDiscoveryInput,
) -> ToolOutput {
    use crate::cooboploop::sources::ObjectiveSourceRegistry;
    let registry = ObjectiveSourceRegistry::init();
    let discovered = registry.discover_all();
    let filtered: Vec<AgentGoal> = if let Some(ref filter) = input.source_type {
        discovered
            .into_iter()
            .filter(|g| {
                let s = format!("{:?}", g.source);
                let filter_lower = filter.to_lowercase();
                s.to_lowercase().contains(&filter_lower)
            })
            .collect()
    } else {
        discovered
    };
    let results: Vec<serde_json::Value> = filtered
        .iter()
        .map(|g| {
            serde_json::json!({
                "id": g.id,
                "title": g.title,
                "source": format!("{:?}", g.source),
                "status": format!("{:?}", g.status),
            })
        })
        .collect();
    ToolOutput::success(serde_json::json!({
        "status": "discovered",
        "count": results.len(),
        "results": results,
        "requested_source_type": input.source_type,
    }))
}

/// Execute cooboploop_evaluate_goal
pub async fn execute_cooboploop_evaluate_goal(
    input: CooboploopEvaluateGoalInput,
    evaluator: &Arc<GoalEvaluator>,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let goal = {
        let queue = match queue.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        queue.goals.get(&input.goal_id).cloned()
    };
    match goal {
        Some(g) => {
            let criteria = evaluator.evaluate(&g);
            let priority_score = evaluator.compute_priority(&g);
            tracing::trace!("Goal evaluated: priority={priority_score}");
            ToolOutput::success(serde_json::json!({
                "status": "evaluated",
                "goal_id": criteria.goal_id,
                "expected_value": criteria.expected_value,
                "probability_of_success": criteria.probability_of_success,
                "urgency": criteria.urgency,
                "risk": criteria.risk,
                "learning_value": criteria.learning_value,
                "strategic_value": criteria.strategic_value,
                "priority_score": criteria.priority_score,
                "required_capabilities": criteria.required_capabilities,
                "deadline": criteria.deadline,
                "resource_cost": criteria.resource_cost.total(),
                "time_cost": criteria.time_cost,
            }))
        }
        None => ToolOutput::success(serde_json::json!({
            "status": "not_found",
            "goal_id": input.goal_id,
        })),
    }
}

/// Execute cooboploop_reprioritize_queue
pub async fn execute_cooboploop_reprioritize_queue(
    queue: &Arc<Mutex<ObjectiveQueue>>,
    evaluator: &Arc<GoalEvaluator>,
) -> ToolOutput {
    let queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut updated = Vec::new();
    for goal in queue.goals.values() {
        let score = evaluator.compute_priority(goal);
        updated.push(serde_json::json!({
            "id": goal.id,
            "title": goal.title,
            "priority_score": score,
        }));
    }
    ToolOutput::success(serde_json::json!({
        "status": "reprioritized",
        "count": updated.len(),
        "updated": updated,
    }))
}

/// Execute cooboploop_set_priority_policy
pub async fn execute_cooboploop_set_priority_policy(
    input: CooboploopSetPriorityPolicyInput,
    evaluator: &Arc<GoalEvaluator>,
    registry: &Arc<Mutex<PriorityPolicyRegistry>>,
) -> ToolOutput {
    let policy_name = input.policy.to_lowercase();
    let new_policy: Box<dyn crate::cooboploop::evaluation::PriorityPolicyTrait> =
        match policy_name.as_str() {
            "conservative" => Box::new(crate::cooboploop::evaluation::ConservativePriorityPolicy),
            "strategic" => Box::new(crate::cooboploop::evaluation::StrategicPolicy),
            "exploration" => Box::new(crate::cooboploop::evaluation::ExplorationPolicy),
            _ => Box::new(crate::cooboploop::evaluation::DefaultPriorityPolicy),
        };
    let sample_goal = crate::cooboploop::sources::HumanInputSource::sample_goal(
        crate::cooboploop::sources::HumanOrigin::UserRequest,
        "Sample",
    );
    let sample_score = new_policy.compute(1.0, &sample_goal);
    let baseline_score = evaluator.compute_priority(&sample_goal);
    let mut reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    reg.set_policy(new_policy);
    let current_policy_name = "policy_active".to_string();
    ToolOutput::success(serde_json::json!({
        "status": "policy_set",
        "requested_policy": input.policy,
        "current_policy": current_policy_name,
        "baseline_score": baseline_score,
        "sample_adjusted_score": sample_score,
    }))
}
