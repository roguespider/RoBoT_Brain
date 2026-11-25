//! Current state evaluation for the CoObOpLoop.

use crate::scheduler::cooboploop::LoopRunner;
use crate::scheduler::cooboploop::idle::IdlePhase;
use crate::scheduler::cooboploop::loop_runner_engine::LoopStage;

/// Evaluate the current cognitive state after each cycle.
pub fn evaluate_current_state(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::EvaluateCurrentState;

    // Get active strategic objectives for evaluation.
    let strategic_objectives = match runner.strategic_registry.lock() {
        Ok(g) => {
            let list = g.list();
            list.to_vec()
        }
        Err(_) => Vec::new(),
    };

    // Evaluate useful work categories.
    let (work_categories, inactivity_result) = runner
        .idle_state
        .evaluate_useful_work(&strategic_objectives);
    if let Some(inactivity) = &inactivity_result {
        tracing::debug!(
            "evaluate_current_state: inactivity event - reason: {}",
            inactivity.reason
        );
    }

    // Check for capability gaps.
    let cap_guard = match runner.capability_registry.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let capability_gaps: Vec<String> = if let Some(ref goal) = runner.selected_goal {
        goal.required_capabilities
            .iter()
            .filter(|cap_id| {
                let cap_id_obj =
                    crate::scheduler::cooboploop::capability::CapabilityId::from_string(cap_id);
                !cap_guard.get(&cap_id_obj).is_some_and(|a| a.level >= 0.5)
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    // Evaluate whether there's useful work or we should wait.
    let queue_depth = runner.objective_queue.len();
    let has_work = !work_categories.is_empty() || queue_depth > 0 || !capability_gaps.is_empty();

    if has_work {
        runner.idle_state.phase = IdlePhase::Active;
        tracing::debug!(
            "evaluate_current_state: useful work available (categories={:?}, queue={}, gaps={:?})",
            work_categories,
            queue_depth,
            capability_gaps
        );
    } else {
        runner.idle_state.enter_wait();
        tracing::debug!("evaluate_current_state: entering wait phase (no useful work)");
    }
}
