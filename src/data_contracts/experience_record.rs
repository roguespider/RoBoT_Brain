/// ExperienceRecord data contract per Architecture Chapter 5.7.
///
/// Experience stores operational history with focus on outcomes.
/// Unlike memory, it tracks what happened and its results.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A record of a completed operational experience.
///
/// Per Architecture Chapter 5.7:
/// id, goal, plan_id, result, success, execution_time, cost,
/// confidence_change, tool_usage, lessons, timestamp
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperienceRecord {
    /// Unique identifier for this experience.
    pub id: String,
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// The goal that initiated this experience.
    pub goal: String,
    /// The plan ID that guided this experience (if any).
    pub plan_id: Option<String>,
    /// Context signature describing the situation for pattern grouping.
    pub context_signature: String,
    /// The outcome description of the experience.
    pub outcome: String,
    /// The outcome result description (alias for outcome, kept for backward compat).
    pub result: String,
    /// Whether the experience was successful.
    pub success: bool,
    /// Execution time in milliseconds.
    pub execution_time_ms: u64,
    /// Computational or resource cost of this experience.
    pub cost: f64,
    /// Change in confidence as a result of this experience.
    pub confidence_change: f32,
    /// Tools used during this experience.
    pub tool_usage: Vec<String>,
    /// Lessons learned from this experience.
    pub lessons: Vec<String>,
}

impl ExperienceRecord {
    /// Create a new experience record with the given goal, context, and result.
    pub fn new(goal: &str, context_signature: &str, outcome: &str, success: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: Metadata::new("experience_engine"),
            goal: goal.to_string(),
            plan_id: None,
            context_signature: context_signature.to_string(),
            outcome: outcome.to_string(),
            result: outcome.to_string(),
            success,
            execution_time_ms: 0,
            cost: 0.0,
            confidence_change: 0.0,
            tool_usage: Vec::new(),
            lessons: Vec::new(),
        }
    }

    /// Mark this experience with a plan ID.
    pub fn with_plan_id(mut self, plan_id: &str) -> Self {
        self.plan_id = Some(plan_id.to_string());
        self
    }

    /// Record the execution time in milliseconds.
    pub fn with_execution_time(mut self, millis: u64) -> Self {
        self.execution_time_ms = millis;
        self
    }

    /// Record a tool used during this experience.
    pub fn with_tool(mut self, tool: &str) -> Self {
        self.tool_usage.push(tool.to_string());
        self
    }

    /// Record a lesson learned.
    pub fn with_lesson(mut self, lesson: &str) -> Self {
        self.lessons.push(lesson.to_string());
        self
    }

    /// Record the confidence change from this experience.
    pub fn with_confidence_change(mut self, delta: f32) -> Self {
        self.confidence_change = delta;
        self
    }

    /// Record the computational cost of this experience.
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }
}

impl Default for ExperienceRecord {
    fn default() -> Self {
        Self {
            id: String::new(),
            metadata: Metadata::default(),
            goal: String::new(),
            plan_id: None,
            context_signature: String::new(),
            outcome: String::new(),
            result: String::new(),
            success: false,
            execution_time_ms: 0,
            cost: 0.0,
            confidence_change: 0.0,
            tool_usage: Vec::new(),
            lessons: Vec::new(),
        }
    }
}
