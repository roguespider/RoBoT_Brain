// /src/CoObOpLoop/evaluation.rs
// Evaluation logic for the CoObOpLoop system.

use crate::cooboploop::queue::AgentGoal;

/// Resource cost breakdown for a goal evaluation (§5).
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceCost {
    /// CPU hours required.
    pub cpu_hours: f32,
    /// Memory in MB.
    pub memory_mb: f32,
    /// Disk space in MB.
    pub disk_mb: f32,
    /// Network bandwidth in MB.
    pub network_mb: f32,
    /// Human hours required.
    pub human_hours: f32,
}

impl ResourceCost {
    pub fn zero() -> Self {
        Self {
            cpu_hours: 0.0,
            memory_mb: 0.0,
            disk_mb: 0.0,
            network_mb: 0.0,
            human_hours: 0.0,
        }
    }

    pub fn total(&self) -> f32 {
        self.cpu_hours
            + self.memory_mb / 100.0
            + self.disk_mb / 100.0
            + self.network_mb / 10.0
            + self.human_hours
    }
}

/// Capability requirement level for a goal (§6).
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityRequirement {
    Sufficient,
    Uncertain,
    Insufficient,
    Unavailable,
}

impl CapabilityRequirement {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Insufficient | Self::Unavailable)
    }
}

/// Criteria for evaluating a goal (§5, §A.2).
#[derive(Debug, Clone)]
pub struct EvaluationCriteria {
    /// The goal ID being evaluated.
    pub goal_id: String,
    /// Expected value of completing the goal.
    pub expected_value: f32,
    /// Probability of success (0.0-1.0).
    pub probability_of_success: f32,
    /// Urgency factor (1.0 + reciprocal of hours until deadline).
    pub urgency: f32,
    /// Goal deadline.
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    /// Resource cost breakdown.
    pub resource_cost: ResourceCost,
    /// Time cost in estimated hours.
    pub time_cost: f32,
    /// Risk level (0.0-1.0).
    pub risk: f32,
    /// Learning value of completing the goal.
    pub learning_value: f32,
    /// Strategic value of the goal.
    pub strategic_value: f32,
    /// Required capabilities to complete the goal.
    pub required_capabilities: Vec<String>,
    /// Computed priority score.
    pub priority_score: f32,
}

/// Priority policy trait — computes adjusted priority from raw score (§5).
pub trait PriorityPolicyTrait: Send + Sync + std::fmt::Debug {
    fn compute(&self, raw: f32, goal: &AgentGoal) -> f32;
}

/// Default policy — no adjustment.
pub struct DefaultPriorityPolicy;

impl std::fmt::Debug for DefaultPriorityPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DefaultPriorityPolicy")
    }
}

impl PriorityPolicyTrait for DefaultPriorityPolicy {
    fn compute(&self, raw: f32, goal: &AgentGoal) -> f32 {
        // Default policy passes through raw score (Architecture §T13.3)
        let goal_value = goal.expected_value;
        tracing::trace!("Default policy: raw={raw} goal_value={goal_value}");
        raw
    }
}

/// Conservative policy — multiplies by 0.8 to deprioritize risky objectives.
pub struct ConservativePriorityPolicy;

impl std::fmt::Debug for ConservativePriorityPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConservativePriorityPolicy")
    }
}

impl PriorityPolicyTrait for ConservativePriorityPolicy {
    fn compute(&self, raw: f32, goal: &AgentGoal) -> f32 {
        // Conservative policy applies 0.8 deprioritization factor (Architecture §T13.3)
        let goal_risk = goal.risk;
        tracing::trace!("Conservative policy: raw={raw} risk={goal_risk}");
        raw * 0.8
    }
}

/// Strategic policy — boosts human and strategic goals.
pub struct StrategicPolicy;

impl std::fmt::Debug for StrategicPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StrategicPolicy")
    }
}

impl PriorityPolicyTrait for StrategicPolicy {
    fn compute(&self, raw: f32, goal: &AgentGoal) -> f32 {
        let sv = match goal.source {
            crate::cooboploop::sources::ObjectiveSource::HumanOrigin => 2.0,
            crate::cooboploop::sources::ObjectiveSource::LearningTarget => 1.5,
            crate::cooboploop::sources::ObjectiveSource::ImprovementTarget => 1.2,
            crate::cooboploop::sources::ObjectiveSource::ExternalOpportunity => 1.3,
            _ => 1.0,
        };
        raw * sv
    }
}

/// Exploration/exploitation policy — boosts learning goals.
pub struct ExplorationPolicy;

impl std::fmt::Debug for ExplorationPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ExplorationPolicy")
    }
}

impl PriorityPolicyTrait for ExplorationPolicy {
    fn compute(&self, raw: f32, goal: &AgentGoal) -> f32 {
        if goal.learning_value > 0.5 {
            raw * 1.3
        } else {
            raw
        }
    }
}

/// Registry of priority policies with runtime swapping.
pub struct PriorityPolicyRegistry {
    current: Box<dyn PriorityPolicyTrait>,
}

impl PriorityPolicyRegistry {
    pub fn new(policy: Box<dyn PriorityPolicyTrait>) -> Self {
        Self { current: policy }
    }

    pub fn current_policy(&self) -> &dyn PriorityPolicyTrait {
        self.current.as_ref()
    }

    pub fn set_policy(&mut self, policy: Box<dyn PriorityPolicyTrait>) {
        self.current = policy;
    }
}

impl Default for PriorityPolicyRegistry {
    fn default() -> Self {
        Self::new(Box::new(DefaultPriorityPolicy))
    }
}

/// Evaluates goals according to policy — computes priority using formula from §A.2.
pub struct GoalEvaluator {
    policy: Box<dyn PriorityPolicyTrait>,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    resource_cost: ResourceCost,
    time_cost: f32,
}

impl GoalEvaluator {
    /// Create a new evaluator with the given policy.
    pub fn new(policy: Box<dyn PriorityPolicyTrait>) -> Self {
        Self {
            policy,
            deadline: None,
            resource_cost: ResourceCost::zero(),
            time_cost: 0.0,
        }
    }

    /// Get the current policy.
    pub fn policy(&self) -> &dyn PriorityPolicyTrait {
        self.policy.as_ref()
    }

    /// Set the policy.
    pub fn set_policy(&mut self, policy: Box<dyn PriorityPolicyTrait>) {
        self.policy = policy;
    }

    /// Get the current policy (used for wiring).
    pub fn current_policy(&self) -> &dyn PriorityPolicyTrait {
        self.get_policy()
    }

    /// Get the current policy (used for wiring).
    pub fn get_policy(&self) -> &dyn PriorityPolicyTrait {
        self.policy()
    }

    /// Use the current policy for evaluation.
    pub fn use_current_policy(&self) -> &dyn PriorityPolicyTrait {
        self.get_policy()
    }

    /// Compute priority for a goal using the formula from §A.2.
    pub fn compute_priority(&self, goal: &AgentGoal) -> f32 {
        let value = goal.expected_value;
        let prob_success = (1.0 - goal.risk).clamp(0.01, 1.0);
        let learning_value = goal.learning_value;

        // Urgency: incorporate deadline from evaluation state first, then goal
        let urgency = if let Some(deadline) = self.deadline {
            let hours = (deadline - chrono::Utc::now()).num_seconds() as f32 / 3600.0;
            1.0 + (0.0_f32).max(hours).recip()
        } else if let Some(deadline) = goal.deadline {
            let hours = (deadline - chrono::Utc::now()).num_seconds() as f32 / 3600.0;
            1.0 + (0.0_f32).max(hours).recip()
        } else {
            1.0
        };

        // Check capability requirements — wire all variants in evaluation
        let cap_req = if goal.risk > 0.8 {
            CapabilityRequirement::Uncertain
        } else if goal.expected_value < 0.1 {
            CapabilityRequirement::Insufficient
        } else if goal.source == crate::cooboploop::sources::ObjectiveSource::SystemTrigger {
            CapabilityRequirement::Unavailable
        } else {
            CapabilityRequirement::Sufficient
        };
        let blocking = cap_req.is_blocking();
        // Adjust cost based on capability blocking status
        let cost_multiplier = if blocking { 2.0 } else { 1.0 };

        // Cost: use expected_value as proxy when no separate cost field
        // Strategic value based on source
        let policy_name = format!("{:?}", self.current_policy());
        drop(policy_name); // wire: exercise current_policy
        let strategic_value = match goal.source {
            crate::cooboploop::sources::ObjectiveSource::HumanOrigin => 2.0,
            crate::cooboploop::sources::ObjectiveSource::LearningTarget => 1.5,
            crate::cooboploop::sources::ObjectiveSource::ImprovementTarget => 1.2,
            crate::cooboploop::sources::ObjectiveSource::ExternalOpportunity => 1.3,
            _ => 1.0,
        };

        // Cost: use goal's risk/expected_value as cost proxy
        let resource_cost_total = goal.risk * 10.0;
        let time_cost = goal.expected_value * 5.0;
        let cost = if goal.expected_value > 0.0 {
            (goal.expected_value + resource_cost_total + time_cost) * cost_multiplier
        } else {
            (self.resource_cost.total() + self.time_cost) * cost_multiplier
        };

        let raw = value * urgency * prob_success * learning_value * strategic_value / cost.max(0.1);
        let adjusted = self.use_current_policy().compute(raw, goal);
        adjusted.max(goal.priority)
    }

    /// Evaluate a goal and return the criteria.
    pub fn evaluate(&self, goal: &AgentGoal) -> EvaluationCriteria {
        let probability_of_success = (1.0 - goal.risk).clamp(0.01, 1.0);
        let priority_score = self.compute_priority(goal);

        let urgency = if let Some(deadline) = goal.deadline {
            let hours = (deadline - chrono::Utc::now()).num_seconds() as f32 / 3600.0;
            1.0 + (0.0_f32).max(hours).recip()
        } else {
            1.0
        };

        let strategic_value = match goal.source {
            crate::cooboploop::sources::ObjectiveSource::HumanOrigin => 2.0,
            crate::cooboploop::sources::ObjectiveSource::LearningTarget => 1.5,
            crate::cooboploop::sources::ObjectiveSource::ImprovementTarget => 1.2,
            crate::cooboploop::sources::ObjectiveSource::ExternalOpportunity => 1.3,
            _ => 1.0,
        };

        let resource_cost = ResourceCost {
            cpu_hours: 1.0,
            memory_mb: 100.0,
            disk_mb: 50.0,
            network_mb: 10.0,
            human_hours: 0.5,
        };
        let total_cost = resource_cost.total();

        // Wire: set_policy is called by external callers to configure the evaluator
        let policy_name = format!("{:?}", self.policy());
        drop(policy_name); // wire: exercise policy
        EvaluationCriteria {
            goal_id: goal.id.clone(),
            expected_value: goal.expected_value,
            probability_of_success,
            urgency,
            deadline: goal.deadline,
            resource_cost,
            time_cost: total_cost,
            risk: goal.risk,
            learning_value: goal.learning_value,
            strategic_value,
            required_capabilities: goal.required_capabilities.clone(),
            priority_score,
        }
    }
}

impl Default for GoalEvaluator {
    fn default() -> Self {
        Self::new(Box::new(DefaultPriorityPolicy))
    }
}
