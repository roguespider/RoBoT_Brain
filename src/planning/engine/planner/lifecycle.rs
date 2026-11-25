//! Plan lifecycle management for the planning engine

use super::Planner;

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::planning::engine::types::{Plan, PlanStatus, PlanStep, StepStatus};
use crate::prompt::{AssemblyParams, PromptBuilder};

impl Planner {
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
        let plan = Self::draft_plan(&goal_str);
        crate::planning::Goal::new(&plan.id, &goal_str).validate()?;

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

    /// Build a plan with the current rule-based planning implementation.
    ///
    /// This synchronous draft is the replaceable CoObOpLoop planning default;
    /// `create_plan` additionally registers the draft and records metrics.
    pub fn draft_plan(goal: &str) -> Plan {
        // Build prompt with layer priorities and token budget (Architecture §17).
        let prompt_builder = PromptBuilder::new();
        let planning_params = AssemblyParams {
            identity: "RoBoT Planning Agent".to_string(),
            rules: "Decompose goals into steps, estimate costs, evaluate alternatives.".to_string(),
            capabilities: "Planning, Memory Retrieval, Knowledge Store, Experience".to_string(),
            context_items: vec![goal.to_string()],
            knowledge_items: Vec::new(),
            experience_items: Vec::new(),
            task_description: goal.to_string(),
            output_format: "Plan with steps, dependencies, confidence".to_string(),
        };
        let constructed_prompt = prompt_builder.build(&planning_params);
        tracing::debug!(
            prompt_layers = ?constructed_prompt.layers,
            token_estimate = constructed_prompt.token_estimate,
            "PromptBuilder assembled planning prompt with {} layers",
            constructed_prompt.layers.len()
        );

        Plan {
            id: Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps: Self::decompose_goal(goal),
            status: PlanStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            knowledge_used: Vec::new(),
            experiences_used: Vec::new(),
            confidence: 0.5,
        }
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

        let mut confidence: f32 = 0.5;

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
        let description = description.into();
        if description.trim().is_empty() {
            return Err(anyhow::anyhow!(
                crate::planning::PlanError::EmptyStepDescription
            ));
        }
        let step = PlanStep {
            id: Uuid::new_v4().to_string(),
            description,
            action: action.into(),
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        };

        let mut plans = self.active_plans.write().await;
        let plan = plans
            .get_mut(plan_id)
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;
        plan.steps.push(step.clone());
        drop(plans);

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
        let description = description.into();
        if description.trim().is_empty() {
            return Err(crate::planning::PlanError::EmptyStepDescription.into());
        }
        let step = PlanStep {
            id: Uuid::new_v4().to_string(),
            description,
            action: action.into(),
            dependencies: Vec::new(),
            status: StepStatus::Pending,
            result: None,
            supporting_knowledge: knowledge_ids,
            past_experiences: experience_ids,
        };

        let mut plans = self.active_plans.write().await;
        let plan = plans
            .get_mut(plan_id)
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;
        plan.steps.push(step.clone());
        plan.confidence = self.calculate_plan_confidence(plan).await;
        drop(plans);

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
        let plan = plans
            .get_mut(plan_id)
            .ok_or_else(|| anyhow::anyhow!("Plan '{}' not found", plan_id))?;
        if !plan.steps.iter().any(|step| step.id == depends_on) {
            anyhow::bail!(
                "Dependency '{}' not found in plan '{}'",
                depends_on,
                plan_id
            );
        }

        // Validate a candidate before committing so rejected edges cannot alter the plan.
        let mut candidate_steps = plan.steps.clone();
        let step = candidate_steps
            .iter_mut()
            .find(|step| step.id == step_id)
            .ok_or_else(|| anyhow::anyhow!("Step '{}' not found in plan '{}'", step_id, plan_id))?;
        if step
            .dependencies
            .iter()
            .any(|dependency| dependency == depends_on)
        {
            return Ok(());
        }
        step.dependencies.push(depends_on.to_string());
        if !crate::planning::validate_no_cycles(&candidate_steps) {
            return Err(crate::planning::PlanError::CircularDependency.into());
        }
        plan.steps = candidate_steps;
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
}
