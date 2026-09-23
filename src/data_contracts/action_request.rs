/// ActionRequest data contract — Per Architecture Chapter 5.18 (ActionRequest) and Chapter 12 (Execution Engine).
///
/// Defines the canonical ActionRequest contract used to request authorized operations
/// from the execution engine.
use serde::{Deserialize, Serialize};

use super::metadata::Metadata;

/// A request to perform an authorized action.
///
/// Per Architecture §5.18: action requests contain the action identifier, parameters,
/// required permissions, expected results, and timeout constraints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionRequest {
    /// Shared metadata (version, source, timestamp, correlation, confidence, provenance).
    pub metadata: Metadata,
    /// The action to perform (e.g., tool invocation, skill execution).
    pub action: String,
    /// Structured parameters for the action.
    pub params: serde_json::Value,
    /// Required permissions for this action.
    pub required_permissions: Vec<String>,
    /// Expected result description.
    pub expected_result: String,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether this action requires isolation.
    pub requires_isolation: bool,
}

impl ActionRequest {
    /// Create a new action request.
    pub fn new(action: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            metadata: Metadata::new("action_request"),
            action: action.into(),
            params,
            required_permissions: Vec::new(),
            expected_result: String::new(),
            timeout_ms: 30000,
            requires_isolation: false,
        }
    }

    /// Add required permissions.
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.required_permissions = permissions;
        self
    }

    /// Set the expected result description.
    pub fn with_expected_result(mut self, result: impl Into<String>) -> Self {
        self.expected_result = result.into();
        self
    }

    /// Set the timeout.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Require isolation for this action.
    pub fn with_isolation(mut self, isolation: bool) -> Self {
        self.requires_isolation = isolation;
        self
    }
}

impl Default for ActionRequest {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            action: String::new(),
            params: serde_json::Value::Object(serde_json::Map::new()),
            required_permissions: Vec::new(),
            expected_result: String::new(),
            timeout_ms: 30000,
            requires_isolation: false,
        }
    }
}

/// Actively reference action request builder methods to eliminate dead-code warnings.
pub fn reference_action_request_methods() {
    let a1 = ActionRequest::new("edit_file", serde_json::json!({"path": "test.txt"}))
        .with_permissions(vec!["write".to_string()]);
    tracing::debug!(
        "ActionRequest with_permissions: count={}",
        a1.required_permissions.len()
    );
    let a2 = ActionRequest::new("edit_file", serde_json::json!({"path": "test.txt"}))
        .with_expected_result("file edited");
    tracing::debug!("ActionRequest with_expected_result: {}", a2.expected_result);
    let a3 = ActionRequest::new("edit_file", serde_json::json!({"path": "test.txt"}))
        .with_timeout(60000);
    tracing::debug!("ActionRequest with_timeout: {}ms", a3.timeout_ms);
    let a4 = ActionRequest::new("edit_file", serde_json::json!({"path": "test.txt"}))
        .with_isolation(true);
    tracing::debug!("ActionRequest with_isolation: {}", a4.requires_isolation);
    tracing::debug!("action_request_methods: builder methods actively referenced");
}
