// src/planner/engine/replanning.rs
//! Replanning functionality for handling plan failures and adaptations

use super::types::{
    Plan, PlanFailureAnalysis, PlanStep, PlanStatus, StepStatus,
};

/// Create a new plan from an existing plan, preserving completed steps
pub fn create_replan(
    existing_plan: &Plan,
    new_plan_id: String,
) -> Plan {
    Plan {
        id: new_plan_id,
        goal: existing_plan.goal.clone(),
        steps: Vec::new(),
        status: PlanStatus::Pending,
        created_at: chrono::Utc::now(),
        completed_at: None,
        knowledge_used: existing_plan.knowledge_used.clone(),
        experiences_used: existing_plan.experiences_used.clone(),
        confidence: existing_plan.confidence * 0.9, // Slight confidence penalty for replanning
    }
}

/// Carry forward completed steps from an existing plan to a new plan
pub fn carry_forward_completed_steps(
    existing_plan: &Plan,
    new_plan: &mut Plan,
) {
    for step in &existing_plan.steps {
        if step.status == StepStatus::Completed {
            new_plan.steps.push(PlanStep {
                id: step.id.clone(),
                description: step.description.clone(),
                action: step.action.clone(),
                dependencies: step.dependencies.clone(),
                status: StepStatus::Completed,
                result: step.result.clone(),
                supporting_knowledge: step.supporting_knowledge.clone(),
                past_experiences: step.past_experiences.clone(),
            });
        }
    }
}

/// Estimate problem complexity from failure rate
pub fn estimate_problem_complexity(failed_step_count: usize) -> f32 {
    
    (failed_step_count as f32 * 0.25).min(1.0)
}

/// Collect completed step IDs from a plan
pub fn collect_completed_step_ids(plan: &Plan) -> Vec<String> {
    plan.steps
        .iter()
        .filter(|s| s.status == StepStatus::Completed)
        .map(|s| s.id.clone())
        .collect()
}

/// Analyze a failed plan to understand what went wrong
pub fn analyze_plan_failure(plan: &Plan, plan_id: &str) -> PlanFailureAnalysis {
    let failed_steps: Vec<_> = plan
        .steps
        .iter()
        .filter(|s| s.status == StepStatus::Failed)
        .collect();

    let mut analysis = PlanFailureAnalysis {
        plan_id: plan_id.to_string(),
        failed_step_count: failed_steps.len(),
        total_steps: plan.steps.len(),
        reasons: Vec::new(),
        suggestions: Vec::new(),
    };

    // Analyze each failed step
    for step in &failed_steps {
        if let Some(ref result) = step.result {
            analysis
                .reasons
                .push(format!("Step '{}' failed: {}", step.description, result));
        }

        // Generate suggestions based on failed step characteristics
        if step.supporting_knowledge.is_empty() {
            analysis.suggestions.push(
                "Consider adding supporting knowledge for step: ".to_string()
                    + &step.description,
            );
        }
        if step.past_experiences.is_empty() {
            analysis.suggestions.push(
                "Look for similar past experiences for step: ".to_string() + &step.description,
            );
        }
    }

    // General suggestions
    if plan.knowledge_used.is_empty() {
        analysis.suggestions.push(
            "This plan has no supporting knowledge. Consider gathering relevant knowledge first.".to_string()
        );
    }
    if plan.confidence < 0.5 {
        analysis.suggestions.push(
            "Plan confidence is low. Consider gathering more evidence before executing."
                .to_string(),
        );
    }

    analysis
}

/// Reset failed steps to ready status for retry
pub fn reset_failed_steps(steps: &mut [PlanStep]) -> usize {
    let mut retried_count = 0;
    for step in steps.iter_mut() {
        if step.status == StepStatus::Failed {
            step.status = StepStatus::Ready;
            step.result = None; // Clear previous failure
            retried_count += 1;
        }
    }
    retried_count
}
