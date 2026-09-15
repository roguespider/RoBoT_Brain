// src/planner/engine/types.rs
//! Core types and data structures for the planning engine

use serde::{Deserialize, Serialize};

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

/// A single step within a plan
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

/// Status of a plan
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PlanStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Status of a plan step
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

/// Reason for replanning
#[derive(Debug, Clone)]
pub enum ReplanReason {
    /// A step in the plan failed
    StepFailed(String),
    /// New knowledge became available
    NewKnowledge(Vec<uuid::Uuid>),
    /// Context changed significantly
    ContextChanged,
    /// User requested replan
    UserRequested,
    /// Better approach discovered
    BetterApproachDiscovered,
    /// Timeout occurred
    Timeout,
}

/// Analysis of why a plan failed
#[derive(Debug, Clone, Default)]
pub struct PlanFailureAnalysis {
    pub plan_id: String,
    pub failed_step_count: usize,
    pub total_steps: usize,
    pub reasons: Vec<String>,
    pub suggestions: Vec<String>,
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

/// Validate that a set of plan steps has no dependency cycles.
pub fn validate_no_cycles(steps: &[PlanStep]) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();

    fn dfs(
        step_id: &str,
        steps: &[PlanStep],
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(step_id.to_string());
        rec_stack.insert(step_id.to_string());

        if let Some(step) = steps.iter().find(|s| s.id == step_id) {
            for dep in &step.dependencies {
                if !visited.contains(dep) {
                    if !dfs(dep, steps, visited, rec_stack) {
                        return false;
                    }
                } else if rec_stack.contains(dep) {
                    return false;
                }
            }
        }

        rec_stack.remove(step_id);
        true
    }

    for step in steps {
        if !visited.contains(&step.id) {
            if !dfs(&step.id, steps, &mut visited, &mut rec_stack) {
                return false;
            }
        }
    }
    true
}

/// Topological sort of plan steps by dependencies.
pub fn topological_sort(steps: &[PlanStep]) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut temp_mark = std::collections::HashSet::new();

    fn visit(
        step_id: &str,
        steps: &[PlanStep],
        visited: &mut std::collections::HashSet<String>,
        temp_mark: &mut std::collections::HashSet<String>,
        result: &mut Vec<String>,
    ) -> bool {
        if temp_mark.contains(step_id) {
            return false;
        }
        if visited.contains(step_id) {
            return true;
        }

        temp_mark.insert(step_id.to_string());
        if let Some(step) = steps.iter().find(|s| s.id == step_id) {
            for dep in &step.dependencies {
                if !visit(dep, steps, visited, temp_mark, result) {
                    return false;
                }
            }
        }
        temp_mark.remove(step_id);
        visited.insert(step_id.to_string());
        result.push(step_id.to_string());
        true
    }

    for step in steps {
        if !visited.contains(&step.id) {
            if !visit(&step.id, steps, &mut visited, &mut temp_mark, &mut result) {
                return None;
            }
        }
    }
    Some(result)
}

/// Planner statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlannerStatistics {
    /// Total number of plans created.
    pub plans_created: usize,
    /// Total number of actions evaluated.
    pub actions_evaluated: usize,
    /// Average action score across evaluated candidates.
    pub avg_score: f32,
}
