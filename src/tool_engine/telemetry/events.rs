//! Events — Per Architecture Chapter 13 (Tool Engine).
//! Event types for telemetry events.

/// Telemetry event types.
#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    /// Tool execution started.
    ToolExecutionStarted { tool_name: String },
    /// Tool execution completed successfully.
    ToolExecutionCompleted { tool_name: String, duration_ms: u64 },
    /// Tool execution failed.
    ToolExecutionFailed { tool_name: String, error: String },
}

/// Record a telemetry event.
pub fn record_event(event: TelemetryEvent) {
    tracing::debug!(event = ?event, "telemetry event recorded");
}
