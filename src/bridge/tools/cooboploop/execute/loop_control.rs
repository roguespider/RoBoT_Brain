use serde_json;
use std::sync::{Arc, Mutex};

use crate::bridge::tools::ToolOutput;
use crate::bridge::tools::cooboploop::inputs::*;

/// Execute cooboploop_record_capability_outcome
pub async fn execute_cooboploop_record_capability_outcome(
    input: CooboploopRecordCapabilityOutcomeInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let mut reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cap_id = crate::cooboploop::capability::CapabilityId::from_string(&input.capability_id);
    let result = if input.success {
        reg.record_success(&cap_id)
    } else {
        reg.record_failure(&cap_id)
    };
    match result {
        Ok(()) => ToolOutput::success(serde_json::json!({
            "message": "Capability outcome recorded",
            "status": "ok",
            "capability_id": input.capability_id,
            "success": input.success,
        })),
        Err(e) => ToolOutput::error(format!("record outcome failed: {e}")),
    }
}

/// Execute cooboploop_get_capability_assessment
pub async fn execute_cooboploop_get_capability_assessment(
    input: CooboploopGetCapabilityAssessmentInput,
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let cap_id = crate::cooboploop::capability::CapabilityId::from_string(&input.capability_id);
    match reg.get(&cap_id) {
        Some(assessment) => ToolOutput::success(serde_json::json!({
            "message": "Capability assessment retrieved",
            "status": "ok",
            "capability_id": input.capability_id,
            "assessment": assessment,
        })),
        None => ToolOutput::success(serde_json::json!({
            "message": "No assessment found for capability",
            "status": "not_found",
            "capability_id": input.capability_id,
            "assessment": serde_json::Value::Null,
        })),
    }
}

/// Execute cooboploop_list_capabilities
pub async fn execute_cooboploop_list_capabilities(
    registry: &Arc<Mutex<crate::cooboploop::capability::CapabilityRegistry>>,
) -> ToolOutput {
    let reg = match registry.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let list: Vec<&crate::cooboploop::capability::CapabilityAssessment> = reg.list();
    let serialized: Vec<serde_json::Value> = list.iter().map(|a| serde_json::json!(a)).collect();
    ToolOutput::success(serde_json::json!({
        "message": "Capabilities listed",
        "status": "ok",
        "count": list.len(),
        "capabilities": serialized,
    }))
}

/// Execute cooboploop_start_loop
pub async fn execute_cooboploop_start_loop(
    input: CooboploopStartLoopInput,
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(v) = input.max_cycles {
        runner.set_max_cycles(v as u32);
    }
    runner.start();
    let status_str = if runner.should_continue() {
        "running"
    } else {
        "stopped"
    };
    ToolOutput::success(serde_json::json!({
        "message": "Loop started",
        "status": status_str,
    }))
}

/// Execute cooboploop_stop_loop
pub async fn execute_cooboploop_stop_loop(
    input: CooboploopStopLoopInput,
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    tracing::debug!("Stopping CoObOpLoop: input={input:?}");
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    runner.stop();
    let status_str = if runner.should_continue() {
        "running"
    } else {
        "stopped"
    };
    ToolOutput::success(serde_json::json!({
        "message": "Loop stopped",
        "status": status_str,
    }))
}

/// Execute cooboploop_get_loop_status
pub async fn execute_cooboploop_get_loop_status(
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let status_str = if runner.should_continue() {
        "running"
    } else {
        "stopped"
    };
    let cycles = runner.cycle_count();
    let max_cycles = runner.max_cycles();
    let stage = format!("{:?}", runner.current_stage());
    let cognitive_stage = format!("{:?}", runner.current_cognitive_stage());
    let summary = runner.cycle_summary();
    let events: &[String] = runner.learning_events();
    // Wire human action handler: include audit summary in loop status
    let human_summary = crate::cooboploop::human::HumanActionHandler::default().audit_summary();
    let state_summary_str = crate::cooboploop::human::HumanActionHandler::default().state_summary();
    ToolOutput::success(serde_json::json!({
        "status": status_str,
        "cycles_completed": cycles,
        "max_cycles": max_cycles,
        "current_stage": stage,
        "cognitive_stage": cognitive_stage,
        "cycle_summary": summary,
        "learning_events": events,
        "human_audit_summary": human_summary,
        "state_summary": state_summary_str,
    }))
}

/// Execute cooboploop_run_single_cycle
pub async fn execute_cooboploop_run_single_cycle(
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Err(error) = runner.run_cycle() {
        return ToolOutput::error(error);
    }
    let cycles = runner.cycle_count();
    ToolOutput::success(serde_json::json!({
        "message": "Cycle completed",
        "status": "ok",
        "cycles_completed": cycles,
    }))
}

/// Execute cooboploop_step_loop
pub async fn execute_cooboploop_step_loop(
    loop_runner: &std::sync::Arc<std::sync::Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
) -> ToolOutput {
    let mut runner = match loop_runner.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if runner.phase == crate::cooboploop::loop_runner::CyclePhase::Wait {
        let resumed = runner.tick_heartbeat();
        if resumed {
            // Heartbeat fired and phase transitioned to Observe — run the cycle now
            // instead of requiring a second step_loop() call.
            if let Err(error) = runner.run_cycle() {
                return ToolOutput::error(error);
            }
        }
        return ToolOutput::success(serde_json::json!({
            "message": if resumed { "Heartbeat resumed and cycle executed" } else { "Heartbeat waiting" },
            "status": if resumed { "resumed" } else { "waiting" },
            "heartbeat_seconds": runner.heartbeat_secs,
            "reevaluation_interval_secs": runner.reevaluation_interval_secs,
            "phase": format!("{:?}", runner.phase),
            "cycles_completed": runner.cycle_count(),
        }));
    }
    if let Err(error) = runner.run_cycle() {
        return ToolOutput::error(error);
    }
    ToolOutput::success(serde_json::json!({
        "message": "Step completed",
        "status": "ok",
        "phase": format!("{:?}", runner.phase),
        "cycles_completed": runner.cycle_count(),
    }))
}

/// Execute cooboploop_run_post_task_evaluation
pub async fn execute_cooboploop_run_post_task_evaluation(
    input: CooboploopRunPostTaskEvaluationInput,
    runner: &Arc<Mutex<crate::cooboploop::loop_runner::LoopRunner>>,
    queue: &Arc<Mutex<crate::cooboploop::queue::ObjectiveQueue>>,
) -> ToolOutput {
    // Look up the goal in the queue to get its real status
    let goal = {
        let q = match queue.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        q.get(&input.goal_id)
    };

    // Compute real evaluation from loop runner state
    let (
        did_succeed,
        verification_confirmed,
        efficiency_score,
        plan_steps_completed,
        plan_steps_total,
    ) = {
        let r = match runner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };

        // The runner holds one evaluation, not evaluation history for every queued goal.
        let goal_selected = goal.is_some()
            && r.selected_goal()
                .is_some_and(|selected| selected.id == input.goal_id);
        let evaluation = r.post_task_evaluation();

        // Check plan step completion
        let (completed, total) = if let Some(plan) = r
            .current_plan()
            .filter(|plan| goal_selected && !plan.steps.is_empty())
        {
            let completed = plan
                .steps
                .iter()
                .filter(|s| {
                    matches!(
                        s.status,
                        crate::planner::engine::types::StepStatus::Completed
                    )
                })
                .count();
            let total = plan.steps.len();
            (completed, total)
        } else {
            (0, 0)
        };

        // Compute efficiency score
        let efficiency = if total > 0 {
            completed as f32 / total as f32
        } else {
            0.0
        };

        let verified = goal_selected && total > 0 && evaluation.verification_confirmed;
        let success = verified && completed == total && evaluation.did_succeed;

        (success, verified, efficiency, completed, total)
    };

    // Get goal status for more accurate evaluation
    let goal_status = goal
        .as_ref()
        .map(|g| format!("{:?}", g.status))
        .unwrap_or_else(|| "not_found".to_string());

    // Build evaluation result with real data
    ToolOutput::success(serde_json::json!({
        "message": "Post-task evaluation completed",
        "goal_id": input.goal_id,
        "goal_status": goal_status,
        "did_succeed": did_succeed,
        "verification_confirmed": verification_confirmed,
        "efficiency_score": efficiency_score,
        "plan_steps_completed": plan_steps_completed,
        "plan_steps_total": plan_steps_total,
        "goal_found": goal.is_some(),
    }))
}
