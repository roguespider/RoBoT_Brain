/// Reflection data contract - Per Architecture Chapter 5.11.
///
/// Reflection evaluates completed execution and transforms it into learning.
/// Per Architecture §5.11: objective achieved, assumptions validated,
/// mistakes discovered, planner evaluation, tool evaluation, suggested improvements.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A reflection on completed execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Reflection {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// Unique identifier for this reflection.
    pub id: String,
    /// Whether the objective was achieved.
    pub objective_achieved: bool,
    /// Assumptions that were validated.
    pub assumptions_validated: Vec<String>,
    /// Mistakes discovered during execution.
    pub mistakes_discovered: Vec<String>,
    /// Evaluation of the planner's performance.
    pub planner_evaluation: String,
    /// Evaluation of tools used.
    pub tool_evaluation: Vec<String>,
    /// Suggested improvements for future execution.
    pub suggested_improvements: Vec<String>,
    /// Timestamp of the reflection.
    pub timestamp: i64,
}

impl Reflection {
    /// Create a new reflection.
    pub fn new(id: impl Into<String>, objective_achieved: bool) -> Self {
        Self {
            metadata: Metadata::new("reflection_system"),
            id: id.into(),
            objective_achieved,
            assumptions_validated: Vec::new(),
            mistakes_discovered: Vec::new(),
            planner_evaluation: String::new(),
            tool_evaluation: Vec::new(),
            suggested_improvements: Vec::new(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Add a validated assumption.
    pub fn with_assumption(mut self, assumption: impl Into<String>) -> Self {
        self.assumptions_validated.push(assumption.into());
        self
    }

    /// Add a discovered mistake.
    pub fn with_mistake(mut self, mistake: impl Into<String>) -> Self {
        self.mistakes_discovered.push(mistake.into());
        self
    }

    /// Set planner evaluation.
    pub fn with_planner_evaluation(mut self, evaluation: impl Into<String>) -> Self {
        self.planner_evaluation = evaluation.into();
        self
    }

    /// Add a tool evaluation.
    pub fn with_tool_evaluation(mut self, evaluation: impl Into<String>) -> Self {
        self.tool_evaluation.push(evaluation.into());
        self
    }

    /// Add a suggested improvement.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggested_improvements.push(suggestion.into());
        self
    }
}

