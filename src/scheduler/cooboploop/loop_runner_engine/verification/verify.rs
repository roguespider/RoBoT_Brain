//! Verification logic for the CoObOpLoop.

use crate::scheduler::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};

/// Verify the outcome of an executed objective cycle.
pub fn verify(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::Verify;

    // Compare execution_result against the original plan to determine outcome.
    let outcome = runner.execution_result.as_deref().unwrap_or("unknown");

    match outcome {
        "completed" => {
            runner.post_task_evaluation.did_succeed = true;
            runner.post_task_evaluation.verification_confirmed = true;
            tracing::info!(
                "verify: execution verified SUCCESS for goal '{}'",
                runner
                    .selected_goal
                    .as_ref()
                    .map(|g| g.title.as_str())
                    .unwrap_or("unknown")
            );
        }
        "failed" => {
            runner.post_task_evaluation.did_succeed = false;
            runner.post_task_evaluation.verification_confirmed = false;
            runner.post_task_evaluation.add_unexpected_problem(format!(
                "Execution of '{}' returned failure",
                runner
                    .selected_goal
                    .as_ref()
                    .map(|g| g.title.as_str())
                    .unwrap_or("unknown")
            ));
            tracing::warn!(
                "verify: execution VERIFICATION FAILED for goal '{}'",
                runner
                    .selected_goal
                    .as_ref()
                    .map(|g| g.title.as_str())
                    .unwrap_or("unknown")
            );
        }
        _ => {
            runner.post_task_evaluation.did_succeed = false;
            runner.post_task_evaluation.verification_confirmed = false;
            runner
                .post_task_evaluation
                .add_unexpected_problem(format!("Unknown execution outcome: '{outcome}'"));
            tracing::debug!("verify: unknown execution outcome '{outcome}' for goal",);
        }
    }
}
