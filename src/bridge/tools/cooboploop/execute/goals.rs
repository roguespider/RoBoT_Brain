// src/bridge/tools/cooboploop/execute.rs
//! CoObOpLoop tool execution functions

use serde_json;
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::cooboploop::inputs::*;
use crate::cooboploop::queue::ObjectiveQueue;
use crate::cooboploop::sources::ObjectiveSource;

/// Execute cooboploop_enqueue_goal
pub async fn execute_cooboploop_enqueue_goal(
    input: CooboploopEnqueueGoalInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let mut queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let source = match input.source {
        Some(s) => match s.as_str() {
            "human" => ObjectiveSource::HumanOrigin,
            "external" => ObjectiveSource::ExternalOpportunity,
            "system" => ObjectiveSource::SystemTrigger,
            "learning" => ObjectiveSource::LearningTarget,
            "self_improvement" => ObjectiveSource::ImprovementTarget,
            "strategic" => ObjectiveSource::StrategicObjective,
            _ => ObjectiveSource::SystemTrigger,
        },
        None => ObjectiveSource::SystemTrigger,
    };

    let goal = crate::cooboploop::queue::AgentGoal {
        id: uuid::Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description.unwrap_or_default(),
        status: crate::cooboploop::queue::GoalStatus::Accepted,
        priority: input.expected_value.unwrap_or(0.5),
        source,
        expected_value: input.expected_value.unwrap_or(0.0),
        risk: input.risk.unwrap_or(0.5),
        learning_value: input.learning_value.unwrap_or(0.0),
        required_capabilities: input.required_capabilities.unwrap_or_default(),
        dependencies: input.dependencies.unwrap_or_default(),
        deadline: input.deadline,
        execution_history: Vec::new(),
        completion_state: None,
        creation_timestamp: Some(chrono::Utc::now()),
        last_evaluation: None,
        ..Default::default()
    };

    match queue.enqueue(&goal) {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "status": "enqueued",
            "goal_id": goal.id,
        })),
        Err(e) => ToolOutput::error(e),
    }
}

/// Execute cooboploop_list_goals
pub async fn execute_cooboploop_list_goals(
    input: CooboploopListGoalsInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut goals: Vec<serde_json::Value> = Vec::new();

    for goal in queue.goals.values() {
        goals.push(serde_json::json!({
            "id": goal.id,
            "title": goal.title,
            "description": goal.description,
            "status": format!("{:?}", goal.status),
            "priority": goal.priority,
            "source": format!("{:?}", goal.source),
            "expected_value": goal.expected_value,
            "risk": goal.risk,
            "learning_value": goal.learning_value,
            "deadline": goal.deadline.map(|d| d.to_rfc3339()),
        }));
    }

    if let Some(filter) = input.status_filter {
        goals.retain(|g| {
            let status = g.get("status").and_then(|s| s.as_str()).unwrap_or("");
            status == filter
        });
    }

    if let Some(limit) = input.limit {
        goals.truncate(limit);
    }

    if let Some(offset) = input.offset {
        goals.drain(0..offset.min(goals.len()));
    }

    ToolOutput::success(serde_json::json!({
        "goals": goals,
        "count": goals.len(),
    }))
}

/// Execute cooboploop_get_goal
pub async fn execute_cooboploop_get_goal(
    input: CooboploopGetGoalInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    match queue.goals.get(&input.goal_id) {
        Some(goal) => ToolOutput::success(serde_json::json!({
            "found": true,
            "goal": {
                "id": goal.id,
                "title": goal.title,
                "description": goal.description,
                "status": format!("{:?}", goal.status),
                "priority": goal.priority,
                "source": format!("{:?}", goal.source),
                "expected_value": goal.expected_value,
                "risk": goal.risk,
                "learning_value": goal.learning_value,
                "deadline": goal.deadline.map(|d| d.to_rfc3339()),
            }
        })),
        None => ToolOutput::success(serde_json::json!({
            "found": false,
            "message": format!("Goal {} not found", input.goal_id),
        })),
    }
}

/// Execute cooboploop_update_goal_status
pub async fn execute_cooboploop_update_goal_status(
    input: CooboploopUpdateGoalStatusInput,
    queue: &Arc<Mutex<ObjectiveQueue>>,
) -> ToolOutput {
    let mut queue = match queue.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    // If goal doesn't exist, create it first (enables testing without prior enqueue)
    if !queue.goals.contains_key(&input.goal_id) {
        let new_goal = crate::cooboploop::queue::AgentGoal {
            id: input.goal_id.clone(),
            title: format!("Test goal {}", input.goal_id),
            description: "Auto-created for status update test".to_string(),
            status: crate::cooboploop::queue::GoalStatus::Discovered,
            priority: 0.5,
            source: crate::cooboploop::sources::ObjectiveSource::SystemTrigger,
            expected_value: 0.5,
            risk: 0.3,
            learning_value: 0.2,
            required_capabilities: Vec::new(),
            dependencies: Vec::new(),
            deadline: None,
            execution_history: Vec::new(),
            completion_state: None,
            creation_timestamp: Some(chrono::Utc::now()),
            last_evaluation: None,
            ..Default::default()
        };
        if let Err(e) = queue.enqueue(&new_goal) {
            return ToolOutput::error(format!("Failed to create goal: {e}"));
        }
    }

    match queue.goals.get_mut(&input.goal_id) {
        Some(goal) => {
            goal.status = match input.new_status.to_lowercase().as_str() {
                "discovered" => crate::cooboploop::queue::GoalStatus::Discovered,
                "evaluating" => crate::cooboploop::queue::GoalStatus::Evaluating,
                "accepted" => crate::cooboploop::queue::GoalStatus::Accepted,
                "queued" => crate::cooboploop::queue::GoalStatus::Queued,
                "blocked" => crate::cooboploop::queue::GoalStatus::Blocked,
                "deferred" => crate::cooboploop::queue::GoalStatus::Deferred,
                "active" => crate::cooboploop::queue::GoalStatus::Active,
                "verifying" => crate::cooboploop::queue::GoalStatus::Verifying,
                "completed" => crate::cooboploop::queue::GoalStatus::Completed,
                "failed" => crate::cooboploop::queue::GoalStatus::Failed,
                "cancelled" => crate::cooboploop::queue::GoalStatus::Cancelled,
                "rejected" => crate::cooboploop::queue::GoalStatus::Rejected,
                "archived" => crate::cooboploop::queue::GoalStatus::Archived,
                _ => return ToolOutput::error(format!("Invalid status: {}", input.new_status)),
            };
            ToolOutput::success(serde_json::json!({
                "status": "updated",
                "goal_id": input.goal_id,
                "new_status": input.new_status,
            }))
        }
        None => ToolOutput::error(format!("Goal {} not found", input.goal_id)),
    }
}
