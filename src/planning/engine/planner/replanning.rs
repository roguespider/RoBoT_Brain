//! Replanning logic for the planning engine

use super::Planner;

use anyhow::Result;
use uuid::Uuid;

use crate::planning::engine::replanning::{
    analyze_plan_failure, carry_forward_completed_steps, collect_completed_step_ids, create_replan,
    estimate_problem_complexity, reset_failed_steps,
};
use crate::planning::engine::types::{
    Plan, PlanFailureAnalysis, PlanStatus, ReplanReason, StepStatus,
};

impl Planner {
    /// Replan an existing plan when circumstances change
    ///
    /// Per Architecture: Replanning occurs when:
    /// - A step fails
    /// - New knowledge becomes available
    /// - Context changes significantly
    /// - A better approach is discovered
    pub async fn replan(&self, plan_id: &str, reason: ReplanReason) -> Result<Option<Plan>> {
        let plans = self.active_plans.read().await;

        let existing_plan = match plans.get(plan_id) {
            Some(plan) => plan.clone(),
            None => return Ok(None),
        };

        let reason_detail = match &reason {
            ReplanReason::StepFailed(step_id) => format!("step {} failed", step_id),
            ReplanReason::NewKnowledge(ids) => format!("{} new knowledge items", ids.len()),
            ReplanReason::ContextChanged => "context changed".to_string(),
            ReplanReason::UserRequested => "user requested".to_string(),
            ReplanReason::BetterApproachDiscovered => "better approach discovered".to_string(),
            ReplanReason::Timeout => "timeout".to_string(),
        };
        tracing::info!("Replanning {} because: {}", plan_id, reason_detail);
        drop(plans);
        self.metrics.increment("planner.replans").await;

        let failed_step_count = existing_plan
            .steps
            .iter()
            .filter(|s| s.status == StepStatus::Failed)
            .count();
        let problem_complexity = estimate_problem_complexity(failed_step_count);

        let use_creativity = self.should_use_creativity(problem_complexity);
        if use_creativity {
            tracing::info!(
                "Using creative approach for replanning (complexity: {:.2})",
                problem_complexity
            );
        }

        let completed_step_ids = collect_completed_step_ids(&existing_plan);

        let mut new_plan = create_replan(&existing_plan, Uuid::new_v4().to_string());
        carry_forward_completed_steps(&existing_plan, &mut new_plan);

        let mut plans = self.active_plans.write().await;
        if let Some(old_plan) = plans.get_mut(plan_id) {
            old_plan.status = PlanStatus::Cancelled;
        }

        plans.insert(new_plan.id.clone(), new_plan.clone());

        tracing::info!(
            "Created new plan {} from {} with {} completed steps",
            new_plan.id,
            plan_id,
            completed_step_ids.len()
        );

        Ok(Some(new_plan))
    }

    /// Retry failed steps in a plan
    ///
    /// Per Architecture: After a step fails, retry with different approach
    pub async fn retry_failed_steps(&self, plan_id: &str) -> Result<usize> {
        let mut plans = self.active_plans.write().await;

        let plan = match plans.get_mut(plan_id) {
            Some(plan) => plan,
            None => return Ok(0),
        };

        let retried_count = reset_failed_steps(&mut plan.steps);

        if retried_count > 0 {
            plan.status = PlanStatus::InProgress;
            self.metrics.increment("planner.step_retries").await;
            tracing::info!("Reset {} failed steps for plan {}", retried_count, plan_id);
        }

        Ok(retried_count)
    }

    /// Adapt a plan based on new knowledge or experience
    ///
    /// Per Architecture: Plans should adapt when new information becomes available
    pub async fn adapt_plan(
        &self,
        plan_id: &str,
        new_knowledge: Vec<uuid::Uuid>,
        new_experiences: Vec<uuid::Uuid>,
    ) -> Result<bool> {
        let mut plans = self.active_plans.write().await;

        let plan = match plans.get_mut(plan_id) {
            Some(plan) => plan,
            None => return Ok(false),
        };

        plan.knowledge_used.extend(new_knowledge);
        plan.experiences_used.extend(new_experiences);

        let policy = self.policy.read().await;
        let knowledge_bonus = if !plan.knowledge_used.is_empty() {
            policy.knowledge_weight * (plan.knowledge_used.len() as f32 * 0.1).min(0.4)
        } else {
            0.0
        };
        let experience_bonus =
            if plan.experiences_used.len() >= policy.min_experience_count as usize {
                policy.experience_weight * (plan.experiences_used.len() as f32 * 0.05).min(0.3)
            } else {
                0.0
            };

        plan.confidence = (0.5_f32 + knowledge_bonus + experience_bonus).clamp(0.0, 1.0);

        self.metrics.increment("planner.plan_adaptations").await;
        tracing::info!(
            "Adapted plan {} with new knowledge and experiences",
            plan_id
        );

        Ok(true)
    }

    /// Analyze a failed plan to understand what went wrong
    pub async fn analyze_failure(&self, plan_id: &str) -> Result<PlanFailureAnalysis> {
        let plans = self.active_plans.read().await;

        let plan = match plans.get(plan_id) {
            Some(plan) => plan,
            None => return Ok(PlanFailureAnalysis::default()),
        };

        let analysis = analyze_plan_failure(plan, plan_id);

        Ok(analysis)
    }
}
