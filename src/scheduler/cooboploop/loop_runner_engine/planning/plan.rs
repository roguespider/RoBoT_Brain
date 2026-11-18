//! Planning logic for the CoObOpLoop.

use crate::planning::engine::types::{Plan, PlanStep, StepStatus};
use crate::scheduler::cooboploop::loop_runner_engine::{LoopRunner, LoopStage};

/// Generate a plan for the selected objective.
pub fn plan(runner: &mut LoopRunner) {
    runner.current_stage = LoopStage::Plan;

    if let Some(ref goal) = runner.selected_goal {
        // Build a real plan from the goal's description and required capabilities.
        let mut steps: Vec<PlanStep> = Vec::new();

        if goal.required_capabilities.is_empty() {
            steps.push(PlanStep {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("Execute: {}", goal.title),
                action: goal.title.clone(),
                dependencies: Vec::new(),
                status: StepStatus::Pending,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        } else {
            for (i, cap) in goal.required_capabilities.iter().enumerate() {
                steps.push(PlanStep {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: format!("Verify capability: {}", cap),
                    action: format!("Verify_{}", cap),
                    dependencies: if i == 0 {
                        Vec::new()
                    } else {
                        vec![steps[i - 1].id.clone()]
                    },
                    status: StepStatus::Pending,
                    result: None,
                    supporting_knowledge: Vec::new(),
                    past_experiences: Vec::new(),
                });
            }
            // Add the main execution step
            steps.push(PlanStep {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("Execute: {}", goal.title),
                action: goal.title.clone(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: StepStatus::Pending,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        let plan = Plan {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.title.clone(),
            steps,
            status: crate::planning::engine::types::PlanStatus::InProgress,
            created_at: chrono::Utc::now(),
            completed_at: None,
            knowledge_used: Vec::new(),
            experiences_used: Vec::new(),
            confidence: 0.7,
        };

        runner.current_plan = Some(plan);
        let step_count = runner.current_plan.as_ref().map_or(0, |p| p.steps.len());
        tracing::info!(
            "plan: generated plan with {step_count} steps for goal '{}'",
            goal.title
        );
    } else {
        tracing::debug!("plan: no selected goal to generate plan for");
    }
}
