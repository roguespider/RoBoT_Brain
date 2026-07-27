// src/planner/planner.rs
//! Core planning engine for task decomposition and execution
//!
//! Per Architecture §2.8, §10:
//! Planning converts knowledge and goals into action.
//! Planning uses accumulated knowledge to make decisions.
//!
//! Per Architecture §5.7 Decision Flow:
//! Goal → Planning → Memory Retrieval → Knowledge Retrieval → Experience Retrieval → Confidence Evaluation → Action Selection → Execution → Outcome Recording



use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::metrics::MetricsCollector;

/// A planned task with decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub status: PlanStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Knowledge IDs used in planning this goal
    pub knowledge_used: Vec<uuid::Uuid>,
    /// Experience IDs that informed this plan
    pub experiences_used: Vec<uuid::Uuid>,
    /// Confidence in this plan based on supporting evidence
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub action: String,
    pub dependencies: Vec<String>,
    pub status: StepStatus,
    pub result: Option<String>,
    /// Knowledge that supports this step
    pub supporting_knowledge: Vec<uuid::Uuid>,
    /// Past experiences that inform this step
    pub past_experiences: Vec<uuid::Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Blocked,
    Ready,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Core planning engine
///
/// Per Architecture §2.8:
/// Planning converts knowledge and goals into action.
pub struct Planner {
    metrics: Arc<MetricsCollector>,
    active_plans: Arc<RwLock<HashMap<String, Plan>>>,
    /// Policy engine for decision making
    policy: Arc<tokio::sync::RwLock<PlannerPolicy>>,
}

/// Planner policy for decision making
///
/// Per Architecture §5.7:
/// Before selecting an action, the system evaluates:
/// - Previous experience
/// - Available knowledge
/// - Confidence levels
/// - Expected outcomes
/// - Potential risks
#[derive(Debug, Clone)]
pub struct PlannerPolicy {
    /// Minimum confidence required to trust knowledge in planning
    pub min_knowledge_confidence: f32,
    /// Minimum experience count to rely on past experiences
    pub min_experience_count: u32,
    /// Weight given to knowledge in decision making
    pub knowledge_weight: f32,
    /// Weight given to experience in decision making
    pub experience_weight: f32,
    /// Weight given to confidence in decision making
    pub confidence_weight: f32,
}

impl Default for PlannerPolicy {
    fn default() -> Self {
        Self {
            min_knowledge_confidence: 0.6,
            min_experience_count: 3,
            knowledge_weight: 0.4,
            experience_weight: 0.3,
            confidence_weight: 0.3,
        }
    }
}

impl Planner {
    /// Create a new planner
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self {
            metrics,
            active_plans: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(tokio::sync::RwLock::new(PlannerPolicy::default())),
        }
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
            confidence: 0.5, // Default confidence
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
        
        // Add knowledge and experiences to plan
        plan.knowledge_used = knowledge_ids;
        plan.experiences_used = experience_ids;
        
        // Calculate plan confidence based on supporting evidence
        plan.confidence = self.calculate_plan_confidence(&plan).await;
        
        // Update in store
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
        
        // Start with base confidence
        let mut confidence = 0.5;
        
        // Factor in knowledge quality
        if !plan.knowledge_used.is_empty() {
            let knowledge_bonus = policy.knowledge_weight * (plan.knowledge_used.len() as f32 * 0.1).min(0.4);
            confidence += knowledge_bonus;
        }
        
        // Factor in experience quality
        if plan.experiences_used.len() >= policy.min_experience_count as usize {
            let experience_bonus = policy.experience_weight * (plan.experiences_used.len() as f32 * 0.05).min(0.3);
            confidence += experience_bonus;
        }
        
        confidence.clamp(0.0, 1.0)
    }

    /// Add a step to a plan
    pub async fn add_step(&self, plan_id: &str, description: impl Into<String>, action: impl Into<String>) -> Result<PlanStep> {
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
            // Recalculate plan confidence
            plan.confidence = self.calculate_plan_confidence(plan).await;
        }

        self.metrics.increment("planner.steps.added").await;

        Ok(step)
    }

    /// Add dependency to a step
    pub async fn add_dependency(&self, plan_id: &str, step_id: &str, depends_on: &str) -> Result<()> {
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
    pub async fn complete_step(&self, plan_id: &str, step_id: &str, result: Option<String>) -> Result<()> {
        let mut plans = self.active_plans.write().await;
        if let Some(plan) = plans.get_mut(plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                step.status = StepStatus::Completed;
                step.result = result;
            }

            // Check if all steps are complete
            let all_complete = plan.steps.iter().all(|s| 
                s.status == StepStatus::Completed || s.status == StepStatus::Skipped
            );
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
        plans.values()
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
    pub async fn select_best_action(&self, actions: Vec<ActionCandidate>) -> Option<ActionCandidate> {
        if actions.is_empty() {
            return None;
        }

        let policy = self.policy.read().await;
        
        // Score each action
        let mut scored: Vec<(ActionCandidate, f32)> = actions
            .into_iter()
            .map(|action| {
                let score = self.score_action(&action, &policy);
                (action, score)
            })
            .collect();
        
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return highest scoring action
        scored.into_iter().next().map(|(action, _)| action)
    }

    /// Score an action candidate based on policy
    fn score_action(&self, action: &ActionCandidate, policy: &PlannerPolicy) -> f32 {
        let mut score = 0.0;
        
        // Factor in supporting knowledge confidence
        if !action.supporting_knowledge.is_empty() {
            let avg_confidence: f32 = action.supporting_knowledge.iter()
                .map(|k| k.confidence)
                .sum::<f32>() / action.supporting_knowledge.len() as f32;
            
            if avg_confidence >= policy.min_knowledge_confidence {
                score += policy.knowledge_weight * avg_confidence;
            }
        }
        
        // Factor in relevant past experiences
        if action.past_experiences.len() >= policy.min_experience_count as usize {
            let success_rate: f32 = action.past_experiences.iter()
                .map(|e| if e.was_successful { 1.0 } else { 0.0 })
                .sum::<f32>() / action.past_experiences.len() as f32;
            score += policy.experience_weight * success_rate;
        }
        
        // Factor in direct confidence
        score += policy.confidence_weight * action.confidence;
        
        score.clamp(0.0, 1.0)
    }

    /// Get plan statistics
    pub async fn get_stats(&self) -> PlannerStats {
        let plans = self.active_plans.read().await;
        
        let mut by_status: std::collections::HashMap<PlanStatus, usize> = std::collections::HashMap::new();
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
            avg_confidence: if count > 0 { total_confidence / count as f32 } else { 0.0 },
            total_knowledge_used: total_knowledge,
            total_experiences_used: total_experiences,
        }
    }
}

/// Action candidate for selection
#[derive(Debug, Clone)]
pub struct ActionCandidate {
    pub id: String,
    pub description: String,
    pub confidence: f32,
    pub supporting_knowledge: Vec<KnowledgeRef>,
    pub past_experiences: Vec<ExperienceRef>,
    pub expected_outcome: Option<String>,
    pub risk_level: RiskLevel,
}

/// Reference to knowledge item
#[derive(Debug, Clone)]
pub struct KnowledgeRef {
    pub id: uuid::Uuid,
    pub confidence: f32,
}

/// Reference to experience
#[derive(Debug, Clone)]
pub struct ExperienceRef {
    pub id: uuid::Uuid,
    pub was_successful: bool,
}

/// Risk level for actions
#[derive(Debug, Clone, Copy)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Planner statistics
#[derive(Debug)]
pub struct PlannerStats {
    pub total_plans: usize,
    pub by_status: std::collections::HashMap<PlanStatus, usize>,
    pub avg_confidence: f32,
    pub total_knowledge_used: usize,
    pub total_experiences_used: usize,
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            metrics: Arc::new(MetricsCollector::new()),
            active_plans: Arc::new(RwLock::new(HashMap::new())),
            policy: Arc::new(tokio::sync::RwLock::new(PlannerPolicy::default())),
        }
    }
}
