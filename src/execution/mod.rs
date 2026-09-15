//! Execution Engine — Execution Request (Architecture Chapter 12.5).
//!
//! The Planning Engine submits a validated execution request.
//! The Execution Engine validates further and executes.
use serde::{Deserialize, Serialize};

use crate::data_contracts::metadata::Metadata;

/// A validated execution request submitted by the Planning Engine.
///
/// Per Architecture Chapter 12.5:
/// execution_id, plan_id, plan_version, goal_id, actions,
/// dependencies, constraints, permissions, budgets, expected_results,
/// checkpoint_policy, metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionRequest {
    /// Unique execution identifier.
    pub execution_id: String,
    /// The plan being executed.
    pub plan_id: String,
    /// Plan version.
    pub plan_version: String,
    /// The goal this execution serves.
    pub goal_id: String,
    /// Actions to execute.
    pub actions: Vec<String>,
    /// Dependencies between actions.
    pub dependencies: Vec<String>,
    /// Execution constraints.
    pub constraints: Vec<String>,
    /// Required permissions.
    pub permissions: Vec<String>,
    /// Resource budgets.
    pub budgets: std::collections::HashMap<String, f64>,
    /// Expected results.
    pub expected_results: Vec<String>,
    /// Checkpoint policy.
    pub checkpoint_policy: String,
    /// Shared metadata.
    pub metadata: Metadata,
}

impl ExecutionRequest {
    /// Create a new execution request.
    pub fn new(plan_id: &str, goal_id: &str) -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            plan_id: plan_id.to_string(),
            plan_version: "1.0".to_string(),
            goal_id: goal_id.to_string(),
            actions: Vec::new(),
            dependencies: Vec::new(),
            constraints: Vec::new(),
            permissions: Vec::new(),
            budgets: std::collections::HashMap::new(),
            expected_results: Vec::new(),
            checkpoint_policy: "standard".to_string(),
            metadata: Metadata::new("execution_engine"),
        }
    }

    /// Add an action.
    pub fn with_action(mut self, action: &str) -> Self {
        self.actions.push(action.to_string());
        self
    }

    /// Add a dependency.
    pub fn with_dependency(mut self, dependency: &str) -> Self {
        self.dependencies.push(dependency.to_string());
        self
    }

    /// Add a constraint.
    pub fn with_constraint(mut self, constraint: &str) -> Self {
        self.constraints.push(constraint.to_string());
        self
    }

    /// Add a permission.
    pub fn with_permission(mut self, permission: &str) -> Self {
        self.permissions.push(permission.to_string());
        self
    }

    /// Set a budget.
    pub fn with_budget(mut self, key: &str, value: f64) -> Self {
        self.budgets.insert(key.to_string(), value);
        self
    }

    /// Add an expected result.
    pub fn with_expected_result(mut self, result: &str) -> Self {
        self.expected_results.push(result.to_string());
        self
    }

    /// Set checkpoint policy.
    pub fn with_checkpoint_policy(mut self, policy: &str) -> Self {
        self.checkpoint_policy = policy.to_string();
        self
    }
}

/// Normalize a raw execution result based on its expected kind.
pub fn normalize_result(raw: &serde_json::Value, kind: OutputKind) -> serde_json::Value {
    match kind {
        OutputKind::Text => serde_json::json!({"text": raw.as_str().unwrap_or("")}),
        OutputKind::Json => raw.clone(),
        OutputKind::Binary => serde_json::json!({"binary": raw.as_str().unwrap_or("")}),
        OutputKind::None => serde_json::Value::Null,
    }
}

/// Expected output kind for execution results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OutputKind {
    #[default]
    None,
    Text,
    Json,
    Binary,
}

/// Target kind for execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetKind {
    Local,
    Network,
    Filesystem,
    Tool,
}

/// Execution step for actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub id: String,
    pub action: String,
    pub params: serde_json::Value,
    pub timeout_ms: u64,
}

/// Retry policy for execution retries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_ms: u64,
}

/// Recovery strategy for failed executions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    Retry,
    Fallback(String),
    Abort,
}

/// Create an execution request from a planner plan.
/// Wiring: `planner/` -> `execution/`
pub fn execution_request_from_plan(
    plan_id: &str,
    goal_id: &str,
    actions: Vec<String>,
) -> ExecutionRequest {
    let mut request = ExecutionRequest::new(plan_id, goal_id);
    for action in actions {
        request = request.with_action(&action);
    }
    request
}

impl Default for ExecutionRequest {
    fn default() -> Self {
        Self {
            execution_id: String::new(),
            plan_id: String::new(),
            plan_version: String::new(),
            goal_id: String::new(),
            actions: Vec::new(),
            dependencies: Vec::new(),
            constraints: Vec::new(),
            permissions: Vec::new(),
            budgets: std::collections::HashMap::new(),
            expected_results: Vec::new(),
            checkpoint_policy: String::new(),
            metadata: Metadata::default(),
        }
    }
}
