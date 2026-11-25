//! Telemetry module — Per Architecture Chapter 13 (Tool Engine).
//! Re-exports telemetry subsystems: tracing, events, and statistics.

pub mod events;
pub mod statistics;
pub mod tracing;

// Re-export telemetry types
pub use events::TelemetryEvent;
pub use events::record_event;
pub use statistics::{
    HealthThresholds, ToolHealthMonitor, ToolHealthStatus, ToolMetricSummary, ToolMetrics,
    get_global_metrics,
};
pub use tracing::{trace_complete, trace_error, trace_execution, trace_start};
