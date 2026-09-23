//! Tool Execution Pipeline — Tool invocation, result handling, audit, security
//! (Architecture Chapter 13.8-13.10 — Tool Engine Execution Pipeline).
//!
//! Per Architecture §13.8-13.10:
//! - Tool invocation through execution engine (§12.17)
//! - Tool result normalization (§12.19) -> experience integration (§12.32)
//! - Tool audit trail (§13.6) -> observability (§27.8)
//! - Tool security (§13.7) -> agent/safety_gate/ (§25.10)
//! - Tool lifecycle: register -> invoke -> result -> audit -> retire

use serde::{Deserialize, Serialize};

/// Tool invocation request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInvocationRequest {
    /// Tool identifier.
    pub tool_id: String,
    /// Caller identifier.
    pub caller: String,
    /// Input parameters.
    pub params: serde_json::Value,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Correlation ID for tracing.
    pub correlation_id: String,
}

impl ToolInvocationRequest {
    /// Create a new invocation request.
    pub fn new(tool_id: &str, caller: &str, params: serde_json::Value) -> Self {
        Self {
            tool_id: tool_id.to_string(),
            caller: caller.to_string(),
            params,
            timeout_ms: 30_000,
            correlation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Tool execution result with audit information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolExecutionResult {
    /// Tool identifier.
    pub tool_id: String,
    /// Caller identifier.
    pub caller: String,
    /// Whether execution succeeded.
    pub success: bool,
    /// Result output.
    pub output: Option<serde_json::Value>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Audit entry ID.
    pub audit_id: String,
    /// Correlation ID.
    pub correlation_id: String,
}

/// Tool audit entry for observability.
/// Per Architecture §13.6 (Tool Audit) and §27.8 (Event Monitoring).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolAuditEntry {
    /// Audit entry ID.
    pub audit_id: String,
    /// Tool identifier.
    pub tool_id: String,
    /// Caller identifier.
    pub caller: String,
    /// Action performed.
    pub action: String,
    /// Timestamp.
    pub timestamp: i64,
    /// Success status.
    pub success: bool,
    /// Correlation ID.
    pub correlation_id: String,
}

impl ToolAuditEntry {
    /// Create a new audit entry.
    pub fn new(
        tool_id: &str,
        caller: &str,
        action: &str,
        success: bool,
        correlation_id: &str,
    ) -> Self {
        Self {
            audit_id: uuid::Uuid::new_v4().to_string(),
            tool_id: tool_id.to_string(),
            caller: caller.to_string(),
            action: action.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            success,
            correlation_id: correlation_id.to_string(),
        }
    }
}

/// Execute a tool invocation through the pipeline.
/// Per Architecture §13.8-13.9: invocation -> authorization -> execution -> result -> audit.
pub fn execute_tool_pipeline(request: &ToolInvocationRequest) -> ToolExecutionResult {
    // Step 1: Authorization check (wiring to security/agent/safety_gate/)
    let authorized = crate::tools::permissions::is_authorized(&request.tool_id, &request.caller);
    if !authorized {
        let audit = ToolAuditEntry::new(
            &request.tool_id,
            &request.caller,
            "invoke_denied",
            false,
            &request.correlation_id,
        );
        tracing::warn!(audit_id = %audit.audit_id, "Tool invocation denied: authorization failed");
        return ToolExecutionResult {
            tool_id: request.tool_id.clone(),
            caller: request.caller.clone(),
            success: false,
            output: None,
            error: Some("Authorization denied".to_string()),
            duration_ms: 0,
            audit_id: audit.audit_id,
            correlation_id: request.correlation_id.clone(),
        };
    }

    // Step 2: Execute through tool registry (wiring to execution/)
    let start = std::time::Instant::now();
    let invocation_result =
        crate::tools::registry::invoke_tool_with_auth(&request.tool_id, &request.caller);
    let duration = start.elapsed().as_millis() as u64;

    // Step 3: Create audit entry (wiring to observability/)
    let audit = ToolAuditEntry::new(
        &request.tool_id,
        &request.caller,
        "invoke",
        invocation_result.is_ok(),
        &request.correlation_id,
    );
    tracing::info!(audit_id = %audit.audit_id, duration_ms = duration, "Tool audit recorded");

    // Step 4: Return result
    match invocation_result {
        Ok(output) => ToolExecutionResult {
            tool_id: request.tool_id.clone(),
            caller: request.caller.clone(),
            success: true,
            output: Some(output),
            error: None,
            duration_ms: duration,
            audit_id: audit.audit_id,
            correlation_id: request.correlation_id.clone(),
        },
        Err(e) => ToolExecutionResult {
            tool_id: request.tool_id.clone(),
            caller: request.caller.clone(),
            success: false,
            output: None,
            error: Some(format!("{:?}", e)),
            duration_ms: duration,
            audit_id: audit.audit_id,
            correlation_id: request.correlation_id.clone(),
        },
    }
}

/// Tool lifecycle: register -> invoke -> result -> audit -> retire.
/// Per Architecture §13.9 (Tool Lifecycle).
pub fn tool_lifecycle(
    tool_id: &str,
    caller: &str,
    params: serde_json::Value,
) -> ToolExecutionResult {
    let request = ToolInvocationRequest::new(tool_id, caller, params);
    execute_tool_pipeline(&request)
}

/// Active reference to tool execution pipeline contracts.
pub fn reference_tool_execution_pipeline() {
    let request =
        ToolInvocationRequest::new("test-tool", "test-caller", serde_json::json!({"param": 1}));
    let result = execute_tool_pipeline(&request);
    let audit = ToolAuditEntry::new("test-tool", "test-caller", "test", result.success, "corr-1");
    tracing::debug!(
        result_success = result.success,
        audit_id = %audit.audit_id,
        duration_ms = result.duration_ms,
        "Tool execution pipeline referenced"
    );
}
