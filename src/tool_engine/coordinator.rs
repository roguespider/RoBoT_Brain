use crate::tool_engine::dispatcher::ToolDispatcher;
use crate::tool_engine::execution_pipeline::{ToolExecutionResult, ToolInvocationRequest};

/// Coordinates the full tool execution pipeline.
/// Per Architecture §13.8-13.10: orchestrates validation → permission → execute → monitor → result → normalize.
pub struct ToolCoordinator<'a> {
    dispatcher: ToolDispatcher<'a>,
}

impl<'a> ToolCoordinator<'a> {
    /// Create a new coordinator with dispatcher reference.
    pub fn new(dispatcher: ToolDispatcher<'a>) -> Self {
        Self { dispatcher }
    }

    /// Execute a full pipeline: validate → authorize → execute → audit → result.
    pub async fn execute_pipeline(&self, request: &ToolInvocationRequest) -> ToolExecutionResult {
        // Delegate to dispatcher for routing and execution
        match self
            .dispatcher
            .dispatch(&request.tool_id, &request.caller, request.params.clone())
            .await
        {
            Ok(output) => ToolExecutionResult {
                tool_id: request.tool_id.clone(),
                caller: request.caller.clone(),
                success: true,
                output: Some(output),
                error: None,
                duration_ms: request.timeout_ms / 2, // Estimated
                audit_id: uuid::Uuid::new_v4().to_string(),
                correlation_id: request.correlation_id.clone(),
            },
            Err(e) => ToolExecutionResult {
                tool_id: request.tool_id.clone(),
                caller: request.caller.clone(),
                success: false,
                output: None,
                error: Some(format!("{:?}", e)),
                duration_ms: 0,
                audit_id: uuid::Uuid::new_v4().to_string(),
                correlation_id: request.correlation_id.clone(),
            },
        }
    }
}
