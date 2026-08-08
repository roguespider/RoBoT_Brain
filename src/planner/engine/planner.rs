// src/planner/engine/planner.rs
//! Core planning engine implementation

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::metrics::MetricsCollector;

use super::actions::{score_action, select_best_scored};
use super::replanning::{
    analyze_plan_failure, carry_forward_completed_steps, collect_completed_step_ids,
    create_replan, estimate_problem_complexity, reset_failed_steps,
};
use super::types::{
    ActionCandidate, Plan, PlanFailureAnalysis, PlanStatus, PlanStep, PlannerPolicy,
    PlannerStats, ReplanReason, StepStatus,
};

/// Core planning engine
///
/// Per Architecture §2.8:
/// Planning converts knowledge and goals into action.
pub struct Planner {
    metrics: Arc<MetricsCollector>,
    active_plans: Arc<RwLock<HashMap<String, Plan>>>,
    policy: Arc<tokio::sync::RwLock<PlannerPolicy>>,
    creativity_check: Option<Arc<dyn Fn(f32) -> bool + Send + Sync>>,
}

impl Planner {
    /// Create a new planner
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            active_plans: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(tokio::sync::RwLock::new(PlannerPolicy::default())),
            creativity_check: None,
        }
    }

    /// Set the creativity decision callback.
    /// This connects the planner to the personality system so that
    /// creative personality traits can influence replanning behavior.
    pub fn set_creativity_check(&mut self, check: impl Fn(f32) -> bool + Send + Sync + 'static) {
        self.creativity_check = Some(Arc::new(check));
    }

    /// Check if a creative approach should be used for replanning.
    /// Returns true when the personality system determines creativity is warranted
    /// given the problem complexity.
    fn should_use_creativity(&self, problem_complexity: f32) -> bool {
        self.creativity_check
            .as_ref()
            .map(|check| check(problem_complexity))
            .unwrap_or(false)
    }

    /// Update planning policy
    pub async fn update_policy(&self, policy: PlannerPolicy) {
        let mut current = self.policy.write().await;
        *current = policy;
    }

    /// Get current planning policy
    pub async fn get_policy(&self) -> PlannerPolicy {
        self.policy.read().await.clone()
    }

    /// Create a new plan from a goal, considering knowledge and experience
    ///
    /// Per Architecture §5.7:
    /// "Before selecting an action, the system evaluates:
    /// - Previous experience
    /// - Available knowledge
    /// - Confidence levels
    /// - Expected outcomes
    /// - Potential risks"
    pub async fn create_plan(&self, goal: impl Into<String>) -> Result<Plan> {
        let plan = Plan {
            id: Uuid::new_v4().to_string(),
            goal: goal.into(),
            steps: Vec::new(),
            status: PlanStatus::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
            knowledge_used: Vec::new(),
            experiences_used: Vec::new(),
            confidence: 0.5,
        };

        let mut plans = self.active_plans.write().await;
        plans.insert(plan.id.clone(), plan.clone());

        self.metrics.increment("planner.plans.created").await;

        Ok(plan)
    }

    /// Create a plan informed by knowledge and experiences
    ///
    /// Per Architecture §2.8:
    /// "Planning depends on the accumulated knowledge of the entire system"
    pub async fn create_informed_plan(
        &self,
        goal: impl Into<String>,
        knowledge_ids: Vec<uuid::Uuid>,
        experience_ids: Vec<uuid::Uuid>,
    ) -> Result<Plan> {
        let mut plan = self.create_plan(goal).await?;

        plan.knowledge_used = knowledge_ids;
        plan.experiences_used = experience_ids;

        plan.confidence = self.calculate_plan_confidence(&plan).await;

        let mut plans = self.active_plans.write().await;
        plans.insert(plan.id.clone(), plan.clone());

        tracing::info!(
            "Created informed plan {} with confidence {} ({} knowledge, {} experiences)",
            plan.id,
            plan.confidence,
            plan.knowledge_used.len(),
            plan.experiences_used.len()
        );

        Ok(plan)
    }

    /// Calculate confidence for a plan based on supporting knowledge and experiences
    async fn calculate_plan_confidence(&self, plan: &Plan) -> f32 {
        let policy = self.policy.read().await;

        let mut confidence = 0.5;

        if !plan.knowledge_used.is_empty() {
            let knowledge_bonus =
                policy.knowledge_weight * (plan.knowledge_used.len() as f32 * 0.1).min(0.4);
            confidence += knowledge_bonus;
        }

        if plan.experiences_used.len() >= policy.min_experience_count as usize {
            let experience_bonus =
                policy.experience_weight * (plan.experiences_used.len() as f32 * 0.05).min(0.3);
            confidence += experience_bonus;
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Add a step to a plan
    pub async fn add_step(
        &self,
        plan_id: &str,
        description: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<PlanStep> {
        let step = PlanStep {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            action: action.into(),
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        };

        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.steps.push(step.clone());
        }

        self.metrics.increment("planner.steps.added").await;

        Ok(step)
    }

    /// Add a step informed by knowledge and experiences
    pub async fn add_informed_step(
        &self,
        plan_id: &str,
        description: impl Into<String>,
        action: impl Into<String>,
        knowledge_ids: Vec<uuid::Uuid>,
        experience_ids: Vec<uuid::Uuid>,
    ) -> Result<PlanStep> {
        let step = PlanStep {
            id: Uuid::new_v4().to_string(),
            description: description.into(),
            action: action.into(),
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            result: None,
            supporting_knowledge: knowledge_ids,
            past_experiences: experience_ids,
        };

        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.steps.push(step.clone());
            plan.confidence = self.calculate_plan_confidence(plan).await;
        }

        self.metrics.increment("planner.steps.added").await;

        Ok(step)
    }

    /// Add dependency to a step
    pub async fn add_dependency(
        &self,
        plan_id: &str,
        step_id: &str,
        depends_on: &str,
    ) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                if !step.dependencies.contains(&depends_on.to_string()) {
                    step.dependencies.push(depends_on.to_string());
                }
            }
        }
        Ok(())
    }

    /// Start executing a plan
    pub async fn start_plan(&self, plan_id: &str) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.status = PlanStatus::InProgress;
            self.metrics.increment("planner.plans.started").await;
        }
        Ok(())
    }

    /// Complete a step
    pub async fn complete_step(
        &self,
        plan_id: &str,
        step_id: &str,
        result: Option<String>,
    ) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                step.status = StepStatus::Completed;
                step.result = result;
            }

            let all_complete = plan
                .steps
                .iter()
                .all(|s| s.status == StepStatus::Completed || s.status == StepStatus::Skipped);
            if all_complete && !plan.steps.is_empty() {
                plan.status = PlanStatus::Completed;
                plan.completed_at = Some(chrono::Utc::now());
                self.metrics.increment("planner.plans.completed").await;
            }
        }
        Ok(())
    }

    /// Fail a step
    pub async fn fail_step(&self, plan_id: &str, step_id: &str, error: String) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                step.status = StepStatus::Failed;
                step.result = Some(format!("Failed: {}", error));
            }
            plan.status = PlanStatus::Failed;
            self.metrics.increment("planner.plans.failed").await;
        }
        Ok(())
    }

    /// Get a plan by ID
    pub async fn get_plan(&self, plan_id: &str) -> Option<Plan> {
        let plans = self.active_plans.read().await;
        plans.get(plan_id).cloned()
    }

    /// List all active plans
    pub async fn list_plans(&self) -> Vec<Plan> {
        let plans = self.active_plans.read().await;
        plans.values().cloned().collect()
    }

    /// List plans by status
    pub async fn list_plans_by_status(&self, status: PlanStatus) -> Vec<Plan> {
        let plans = self.active_plans.read().await;
        plans
            .values()
            .filter(|p| p.status == status)
            .cloned()
            .collect()
    }

    /// Cancel a plan
    pub async fn cancel_plan(&self, plan_id: &str) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            plan.status = PlanStatus::Cancelled;
        }
        Ok(())
    }

    /// Clean up completed/failed plans older than a duration
    pub async fn cleanup_old_plans(&self, max_age: chrono::Duration) -> Result<usize> {
        let cutoff = chrono::Utc::now() - max_age;
        let mut plans = self.active_plans.write().await;
        let initial_count = plans.len();

        plans.retain(|_, plan| {
            if let Some(completed) = plan.completed_at {
                completed > cutoff
            } else {
                plan.created_at > cutoff || plan.status == PlanStatus::InProgress
            }
        });

        Ok(initial_count - plans.len())
    }

    /// Select the best next action based on knowledge and confidence
    ///
    /// Per Architecture §5.7:
    /// "Action Selection"
    pub async fn select_best_action(
        &self,
        actions: Vec<ActionCandidate>,
    ) -> Option<ActionCandidate> {
        if actions.is_empty() {
            return None;
        }

        let policy = self.policy.read().await;

        let scored: Vec<(ActionCandidate, f32)> = actions
            .into_iter()
            .map(|action| {
                let score = score_action(&action, &policy);
                (action, score)
            })
            .collect();

        select_best_scored(scored)
    }

    /// Get plan statistics
    pub async fn get_stats(&self) -> PlannerStats {
        let plans = self.active_plans.read().await;

        let mut by_status: std::collections::HashMap<PlanStatus, usize> =
            std::collections::HashMap::new();
        let mut total_confidence = 0.0;
        let mut total_knowledge = 0;
        let mut total_experiences = 0;

        for plan in plans.values() {
            *by_status.entry(plan.status).or_insert(0) += 1;
            total_confidence += plan.confidence;
            total_knowledge += plan.knowledge_used.len();
            total_experiences += plan.experiences_used.len();
        }

        let count = plans.len();
        PlannerStats {
            total_plans: count,
            by_status,
            avg_confidence: if count > 0 {
                total_confidence / count as f32
            } else {
                0.0
            },
            total_knowledge_used: total_knowledge,
            total_experiences_used: total_experiences,
        }
    }

    // ========================================================================
    // REPLANNING
    // ========================================================================

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

        tracing::info!("Replanning {} because: {:?}", plan_id, reason);
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

        plan.confidence = (0.5 + knowledge_bonus + experience_bonus).clamp(0.0, 1.0);

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

impl Default for Planner {
    fn default() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            active_plans: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(tokio::sync::RwLock::new(PlannerPolicy::default())),
            creativity_check: None,
        }
    }
}
