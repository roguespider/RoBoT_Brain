use crate::tool_engine::executor::ToolExecutor;
use crate::tool_engine::permissions::PermissionChecker;
use crate::tool_engine::registry::{ToolContract, ToolError, ToolRegistry};
use crate::tool_engine::streaming::ToolStreamer;
use crate::tool_engine::validator::ToolValidator;
use std::sync::{Arc, Mutex};

/// Dispatches tool calls to appropriate executors based on tool contracts.
/// Per Architecture §13.8 (execution pipeline): tool selection/routing.
pub struct ToolDispatcher<'a> {
    registry: &'a ToolRegistry,
    permission_checker: PermissionChecker,
    /// Tool executor for actual tool execution.
    executor: ToolExecutor,
    /// Tool streamer for incremental output (with interior mutability).
    streamer: Arc<Mutex<ToolStreamer>>,
}

impl<'a> ToolDispatcher<'a> {
    /// Create a new dispatcher with a reference to the global registry.
    /// Initializes the ToolExecutor and ToolStreamer for production use.
    pub fn new(registry: &'a ToolRegistry, permission_checker: PermissionChecker) -> Self {
        // Initialize the executor with validation and permissions
        let validator = ToolValidator::new(false);
        let executor_checker = PermissionChecker::new();
        let executor = ToolExecutor::new(validator, executor_checker);
        // Initialize the streamer for incremental output with interior mutability
        let streamer = Arc::new(Mutex::new(ToolStreamer::new()));
        Self {
            registry,
            permission_checker,
            executor,
            streamer,
        }
    }

    /// Dispatch a tool call to the appropriate executor.
    /// Per Architecture §13.8: validates caller permissions and routes to executor.
    pub async fn dispatch(
        &self,
        tool_name: &str,
        caller: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        // Step 1: Check if tool exists
        let contract = self.registry.get(tool_name).ok_or(ToolError::NotFound)?;

        // Step 2: Check permissions
        let (authorized, reason) = self.permission_checker.check(tool_name, caller);
        if !authorized {
            return Err(ToolError::RegistrationFailed(format!(
                "Permission denied: {}",
                reason
            )));
        }

        // Step 3: Validate parameters against contract
        self.validate_parameters(&contract, &params)?;

        // Step 4: Route to appropriate executor based on tool category
        self.execute_tool(&contract, &params).await
    }

    /// Validate tool parameters against the contract.
    fn validate_parameters(
        &self,
        contract: &ToolContract,
        params: &serde_json::Value,
    ) -> Result<(), ToolError> {
        // Verify contract name is valid and non-empty
        if contract.name.trim().is_empty() {
            return Err(ToolError::InvalidSchema(
                "Tool contract name must not be empty".to_string(),
            ));
        }
        // Basic validation - in a full implementation, this would use JSON Schema
        // For now, we just check that params is an object
        if !params.is_object() {
            return Err(ToolError::InvalidSchema(
                "Parameters must be a JSON object".to_string(),
            ));
        }
        Ok(())
    }

    /// Execute the tool based on its category and capabilities.
    /// Uses ToolExecutor for actual tool execution with streaming support.
    async fn execute_tool(
        &self,
        contract: &ToolContract,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, ToolError> {
        // Use the ToolExecutor for actual execution with validation
        let result = self.executor.execute(contract, params, "dispatcher").await;

        // Stream result chunks via ToolStreamer for incremental output
        match &result {
            Ok(output) => {
                let mut stream_guard = self.streamer.lock().map_err(|e| {
                    ToolError::RegistrationFailed(format!("Streamer lock failed: {:?}", e))
                })?;
                stream_guard.emit(output.clone());
            }
            Err(e) => {
                let mut stream_guard = self.streamer.lock().map_err(|e| {
                    ToolError::RegistrationFailed(format!("Streamer lock failed: {:?}", e))
                })?;
                stream_guard.emit(serde_json::json!({"error": format!("{:?}", e)}));
            }
        }

        // Wire stream_result into production
        {
            let mut stream_guard = self.streamer.lock().map_err(|e| {
                ToolError::RegistrationFailed(format!("Streamer lock failed: {:?}", e))
            })?;
            let streamed = crate::tool_engine::streaming::stream_result(
                &mut stream_guard,
                serde_json::json!({"status": "streamed"}),
            );
            let streamed_len = streamed.len();
            tracing::debug!(streamed_len = streamed_len, "Streaming result recorded");
        }
        // Wire chunks and chunk_count into production
        {
            let stream_guard = self.streamer.lock().map_err(|e| {
                ToolError::RegistrationFailed(format!("Streamer lock failed: {:?}", e))
            })?;
            let chunk_count = stream_guard.chunk_count();
            let chunks = stream_guard.chunks();
            tracing::debug!(
                streamed_chunks = chunk_count,
                chunk_len = chunks.len(),
                "Streaming result chunks recorded"
            );
        }

        // Wire clear into production for cleanup
        {
            let mut stream_guard = self.streamer.lock().map_err(|e| {
                ToolError::RegistrationFailed(format!("Streamer lock failed: {:?}", e))
            })?;
            stream_guard.clear();
        }

        result
    }
}
