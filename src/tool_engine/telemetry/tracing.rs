//! Tracing — Per Architecture Chapter 13 (Tool Engine).
//! Tracing utilities for tool execution observability.

/// Trace a tool execution.
pub fn trace_execution(tool_name: &str, action: &str) {
    tracing::debug!(tool = tool_name, action, "tool execution traced");
}

/// Trace a tool error.
pub fn trace_error(tool_name: &str, error: &str) {
    tracing::error!(tool = tool_name, error, "tool execution error");
}

/// Trace a tool start.
pub fn trace_start(tool_name: &str) {
    tracing::debug!(tool = tool_name, "tool execution started");
}

/// Trace a tool completion.
pub fn trace_complete(tool_name: &str, duration_ms: u64) {
    tracing::debug!(tool = tool_name, duration_ms, "tool execution completed");
}
