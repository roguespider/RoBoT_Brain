// src/planner/mod.rs
//! Planning and decision-making module
//!
//! Per Architecture Chapter 11 - Planning Engine:
//! - Goal creation with validation
//! - Step generation and decomposition
//! - Dependency-aware task graphs
//! - Planning strategy selection
//! - Candidate plan generation and evaluation

use chrono::Utc;

/// Error types for planning operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanError {
    /// Goal description is empty.
    EmptyDescription,
    /// Priority is out of valid range (0..=10).
    InvalidPriority,
    /// Deadline is in the past.
    DeadlineInPast,
    /// Step description is empty.
    EmptyStepDescription,
    /// Circular dependency detected in steps.
    CircularDependency,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::EmptyDescription => write!(f, "Goal description must be non-empty"),
            PlanError::InvalidPriority => write!(f, "Priority must be between 0 and 10"),
            PlanError::DeadlineInPast => write!(f, "Deadline must be in the future"),
            PlanError::EmptyStepDescription => write!(f, "Step description must be non-empty"),
            PlanError::CircularDependency => {
                write!(f, "Circular dependency detected in plan steps")
            }
        }
    }
}

/// A goal to be planned and executed.
pub struct Goal {
    /// Unique identifier for the goal.
    pub id: String,
    /// Human-readable description of what needs to be achieved.
    pub description: String,
    /// Priority level (0 = lowest, 10 = highest).
    pub priority: u8,
    /// Optional deadline as Unix timestamp (seconds since epoch).
    pub deadline: Option<i64>,
    /// Whether the goal has been completed.
    pub completed: bool,
}

impl Goal {
    /// Create a new goal with the given id and description.
    pub fn new(id: impl Into<String>, description: impl Into<String>, priority: u8) -> Self {
        Goal {
            id: id.into(),
            description: description.into(),
            priority,
            deadline: None,
            completed: false,
        }
    }
}

/// Validate a goal, ensuring all constraints are met.
pub fn validate_goal(g: &Goal) -> Result<(), PlanError> {
    if g.description.trim().is_empty() {
        return Err(PlanError::EmptyDescription);
    }
    if g.priority > 10 {
        return Err(PlanError::InvalidPriority);
    }
    if let Some(deadline) = g.deadline {
        let now = Utc::now().timestamp();
        if deadline < now {
            return Err(PlanError::DeadlineInPast);
        }
    }
    Ok(())
}

/// Generate plan steps from a goal description.
///
/// Per Architecture Chapter 11.2: produces a skeleton plan with steps
/// parsed from the goal's action verbs and keywords.
pub fn generate_steps(goal: &Goal) -> Vec<crate::planner::engine::types::PlanStep> {
    let lower = goal.description.to_lowercase();
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

    let mut step_num = 0u32;

    if wants_search {
        steps.push(crate::planner::engine::types::PlanStep {
            id: format!("step-{}", step_num),
            description: format!("Search for: {}", goal.description),
            action: "search".to_string(),
            dependencies: Vec::new(),
            status: crate::planner::engine::types::StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        });
        step_num += 1;
    }

    if wants_store {
        steps.push(crate::planner::engine::types::PlanStep {
            id: format!("step-{}", step_num),
            description: format!("Store result: {}", goal.description),
            action: "store".to_string(),
            dependencies: Vec::new(),
            status: crate::planner::engine::types::StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        });
        step_num += 1;
    }

    if wants_knowledge {
        steps.push(crate::planner::engine::types::PlanStep {
            id: format!("step-{}", step_num),
            description: format!("Extract knowledge: {}", goal.description),
            action: "learn".to_string(),
            dependencies: Vec::new(),
            status: crate::planner::engine::types::StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        });
        step_num += 1;
    }

    if wants_analyze {
        steps.push(crate::planner::engine::types::PlanStep {
            id: format!("step-{}", step_num),
            description: format!("Analyze: {}", goal.description),
            action: "analyze".to_string(),
            dependencies: Vec::new(),
            status: crate::planner::engine::types::StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        });
        step_num += 1;
    }

    if wants_plan {
        steps.push(crate::planner::engine::types::PlanStep {
            id: format!("step-{}", step_num),
            description: format!("Plan: {}", goal.description),
            action: "plan".to_string(),
            dependencies: Vec::new(),
            status: crate::planner::engine::types::StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        });
        step_num += 1;
    }

    // If no keywords matched, produce a default step.
    if steps.is_empty() {
        steps.push(crate::planner::engine::types::PlanStep {
            id: format!("step-{}", step_num),
            description: goal.description.clone(),
            action: "execute".to_string(),
            dependencies: Vec::new(),
            status: crate::planner::engine::types::StepStatus::Pending,
            result: None,
            supporting_knowledge: Vec::new(),
            past_experiences: Vec::new(),
        });
    }

    steps
}

pub mod engine;
pub mod policy;

pub use engine::Planner;
pub use policy::PolicyEngine;
