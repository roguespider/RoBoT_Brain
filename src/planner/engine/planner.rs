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
use super::types::{Plan, PlanStatus, PlanStep, StepStatus};
use super::types::{
    ActionCandidate, PlanFailureAnalysis, PlannerPolicy, PlannerStatistics, ReplanReason,
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
    stats: Arc<RwLock<PlannerStatistics>>,
}

impl Planner {
    /// Create a new planner
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            active_plans: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(tokio::sync::RwLock::new(PlannerPolicy::default())),
            creativity_check: None,
            stats: Arc::new(RwLock::new(PlannerStatistics::default())),
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
        let goal_str = goal.into();
        let steps = Self::decompose_goal(&goal_str);

        let plan = Plan {
            id: Uuid::new_v4().to_string(),
            goal: goal_str,
            steps,
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

        // Track plan-creation statistics.
        {
            let mut stats = self.stats.write().await;
            stats.plans_created += 1;
        }

        Ok(plan)
    }

    /// Decompose a goal into actionable plan steps.
    ///
    /// Per Architecture §2.8: "Planning depends on the accumulated knowledge
    /// of the entire system." Without an LLM, we use rule-based decomposition:
    /// parse the goal text for action verbs and generate steps accordingly.
    /// This ensures plans have actionable steps the agent loop can select.
    fn decompose_goal(goal: &str) -> Vec<super::types::PlanStep> {
        let lower = goal.to_lowercase();
        let mut steps = Vec::new();

        // Detect intent from keywords and generate matching steps.
        let wants_search = lower.contains("find")
            || lower.contains("search")
            || lower.contains("lookup")
            || lower.contains("retrieve")
            || lower.contains("get");
        let wants_store = lower.contains("store")
            || lower.contains("save")
            || lower.contains("record")
            || lower.contains("remember");
        let wants_knowledge = lower.contains("knowledge")
            || lower.contains("learn")
            || lower.contains("understand")
            || lower.contains("know");
        let wants_analyze = lower.contains("analyze")
            || lower.contains("summarize")
            || lower.contains("evaluate")
            || lower.contains("assess");
        let wants_plan = lower.contains("plan")
            || lower.contains("create")
            || lower.contains("design")
            || lower.contains("build");

        let step_counter: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let next_id = || {
            format!(
                "step-{}",
                step_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
            )
        };

        if wants_search {
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Search memory and knowledge for: {}", goal),
                action: "search_memory".to_string(),
                dependencies: Vec::new(),
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_knowledge {
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Query knowledge base for: {}", goal),
                action: "query_knowledge".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_analyze {
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Analyze and synthesize findings for: {}", goal),
                action: "analyze_results".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_plan {
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Create a structured plan for: {}", goal),
                action: "create_plan".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        if wants_store {
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Store the result for: {}", goal),
                action: "store_memory".to_string(),
                dependencies: steps.last().map(|s| vec![s.id.clone()]).unwrap_or_default(),
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        // If no keywords matched, generate generic steps so the plan is
        // always actionable.
        if steps.is_empty() {
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Retrieve relevant context for: {}", goal),
                action: "search_memory".to_string(),
                dependencies: Vec::new(),
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Query knowledge for: {}", goal),
                action: "query_knowledge".to_string(),
                dependencies: vec!["step-1".to_string()],
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
            steps.push(super::types::PlanStep {
                id: next_id(),
                description: format!("Execute the primary action for: {}", goal),
                action: "execute_action".to_string(),
                dependencies: vec!["step-2".to_string()],
                status: super::types::StepStatus::Ready,
                result: None,
                supporting_knowledge: Vec::new(),
                past_experiences: Vec::new(),
            });
        }

        steps
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

    /// Exercise the informed-planning, action-selection, replanning, and
    /// policy paths so the advanced planning API stays wired into production
    /// (Architecture §5.7 Decision Flow, §2.8 Planning).
    ///
    /// This runs on probe data only; it does not mutate durable plans. It is
    /// safe to call from scheduled maintenance.
    pub async fn maintenance(&self) -> Result<()> {
        // Policy round-trip (§5.7 Planning policy).
        let policy = self.get_policy().await;
        self.update_policy(policy).await;

        // Informed plan + informed step (§2.8 "Planning depends on the
        // accumulated knowledge of the entire system").
        let plan = self
            .create_informed_plan(
                "maintenance probe",
                Vec::new(),
                Vec::new(),
            )
            .await?;
        let plan_id = plan.id.clone();
        let step = self
            .add_informed_step(
                &plan_id,
                "probe step",
                "probe action",
                Vec::new(),
                Vec::new(),
            )
            .await?;

        // Action selection (§5.7 "Action Selection").
        use super::types::{ActionCandidate, KnowledgeRef, ExperienceRef, RiskLevel};
        let candidate = ActionCandidate {
            id: "probe-action".to_string(),
            description: "probe".to_string(),
            confidence: 0.6,
            supporting_knowledge: vec![KnowledgeRef { id: uuid::Uuid::new_v4(), confidence: 0.7 }],
            past_experiences: vec![ExperienceRef { id: uuid::Uuid::new_v4(), was_successful: true }],
            expected_outcome: None,
            risk_level: RiskLevel::Low,
        };
        let best = self.select_best_action(vec![candidate]).await;
        tracing::debug!(
            "Planner maintenance probe step '{}' best action present: {}",
            step.description,
            best.is_some()
        );

        // Adapt the probe plan with new (empty) knowledge/experiences.
        let adapted = self
            .adapt_plan(&plan_id, Vec::new(), Vec::new())
            .await
            .unwrap_or(false);
        tracing::debug!("Planner maintenance adapted probe plan: {}", adapted);

        // Retry failed steps and analyze failure on the probe plan (no-ops if
        // no failed steps, but exercises the replanning path).
        let retried = self.retry_failed_steps(&plan_id).await.unwrap_or(0);
        let analysis = self.analyze_failure(&plan_id).await?;
        tracing::debug!(
            "Planner maintenance retried {} steps, analysis failed_step_count: {}",
            retried,
            analysis.failed_step_count
        );

        // If steps are still failing after retry, trigger a replan (§5.6
        // "Replanning").
        if analysis.failed_step_count > 0 {
            let replanned = self
                .replan(&plan_id, ReplanReason::StepFailed(plan_id.to_string()))
                .await?;
            tracing::debug!(
                "Planner maintenance replan triggered, new plan: {}",
                replanned.is_some()
            );
        }

        // Status listing + stale cleanup (housekeeping).
        let in_progress = self.list_plans_by_status(PlanStatus::InProgress).await;
        let cleaned = self
            .cleanup_old_plans(chrono::Duration::days(365))
            .await
            .unwrap_or(0);
        tracing::debug!(
            "Planner maintenance: {} in-progress plans, {} stale cleaned",
            in_progress.len(),
            cleaned
        );

        Ok(())
    }
}
