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
#[cfg(test)]
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

#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
pub struct PlanFailureAnalysis {
    pub plan_id: String,
    pub failed_step_count: usize,
    pub total_steps: usize,
    pub reasons: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Action candidate for selection
#[derive(Debug, Clone)]
#[cfg(test)]
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
#[cfg(test)]
pub struct KnowledgeRef {
    pub id: uuid::Uuid,
    pub confidence: f32,
}

/// Reference to experience
#[derive(Debug, Clone)]
#[cfg(test)]
pub struct ExperienceRef {
    pub id: uuid::Uuid,
    pub was_successful: bool,
}

/// Risk level for actions
#[derive(Debug, Clone, Copy)]
#[cfg(test)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Planner statistics
#[derive(Debug)]
#[cfg(test)]
pub struct PlannerStats {
    pub total_plans: usize,
    pub by_status: std::collections::HashMap<PlanStatus, usize>,
    pub avg_confidence: f32,
    pub total_knowledge_used: usize,
    pub total_experiences_used: usize,
}
