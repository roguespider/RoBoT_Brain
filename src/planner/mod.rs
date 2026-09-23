// src/planner/mod.rs
//! Planning and decision-making module
//!
//! Per Architecture Chapter 11 - Planning Engine:
//! - Goal creation with validation
//! - Step generation and decomposition
//! - Dependency-aware task graphs
//! - Planning strategy selection
//! - Candidate plan generation and evaluation

pub mod engine;
pub mod policy;

pub use engine::Planner;
pub use engine::types::PlanStep;
pub use policy::PolicyEngine;

/// A planning goal with validation rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: u8,
    pub deadline: Option<i64>,
    pub completed: bool,
}

impl Goal {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            priority: 5,
            deadline: None,
            completed: false,
        }
    }

    /// Validate goal constraints before creating a plan.
    pub fn validate(&self) -> Result<(), PlanError> {
        if self.description.trim().is_empty() {
            return Err(PlanError::EmptyDescription);
        }
        if self.priority > 10 {
            return Err(PlanError::InvalidPriority);
        }
        if self
            .deadline
            .is_some_and(|deadline| deadline < chrono::Utc::now().timestamp())
        {
            return Err(PlanError::DeadlineInPast);
        }
        Ok(())
    }
}

/// Planning errors for validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    EmptyDescription,
    InvalidPriority,
    DeadlineInPast,
    EmptyStepDescription,
    CircularDependency,
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::EmptyDescription => write!(f, "goal description is empty"),
            PlanError::InvalidPriority => write!(f, "priority must be between 0 and 10"),
            PlanError::DeadlineInPast => write!(f, "deadline is in the past"),
            PlanError::EmptyStepDescription => write!(f, "step description is empty"),
            PlanError::CircularDependency => write!(f, "circular dependency detected"),
        }
    }
}

impl std::error::Error for PlanError {}

/// Planning strategy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningStrategy {
    Sequential,
    Parallel,
    Greedy,
}

/// Select a planning strategy based on goal characteristics
pub fn select_strategy(goal: &Goal) -> PlanningStrategy {
    if goal.priority > 7 {
        PlanningStrategy::Greedy
    } else if goal.description.contains("parallel") || goal.description.contains("simultaneous") {
        PlanningStrategy::Parallel
    } else {
        PlanningStrategy::Sequential
    }
}

/// Validate that a set of plan steps has no circular dependencies (DFS)
pub fn validate_no_cycles(steps: &[PlanStep]) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut rec_stack = std::collections::HashSet::new();

    fn has_cycle(
        step_id: &str,
        steps: &[PlanStep],
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(step_id.to_string());
        rec_stack.insert(step_id.to_string());

        let step = steps.iter().find(|s| s.id == step_id);
        if let Some(s) = step {
            for dep in &s.dependencies {
                if !visited.contains(dep) {
                    if has_cycle(dep, steps, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(step_id);
        false
    }

    for step in steps {
        if !visited.contains(&step.id) && has_cycle(&step.id, steps, &mut visited, &mut rec_stack) {
            return false;
        }
    }
    true
}

/// Topological sort of plan steps using Kahn's algorithm
pub fn topological_sort(steps: &[PlanStep]) -> Option<Vec<String>> {
    let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    for step in steps {
        in_degree.insert(step.id.clone(), 0);
        adj.insert(step.id.clone(), Vec::new());
    }

    for step in steps {
        for dep in &step.dependencies {
            if adj.contains_key(dep) {
                if let Some(adj_list) = adj.get_mut(dep) {
                    adj_list.push(step.id.clone());
                }
                if let Some(deg) = in_degree.get_mut(&step.id) {
                    *deg += 1;
                }
            }
        }
    }

    let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    for (id, deg) in &in_degree {
        if *deg == 0 {
            queue.push_back(id.clone());
        }
    }

    let mut result = Vec::new();
    while let Some(current) = queue.pop_front() {
        result.push(current.clone());
        if let Some(neighbors) = adj.get(&current) {
            for neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    if result.len() == steps.len() {
        Some(result)
    } else {
        None
    }
}
