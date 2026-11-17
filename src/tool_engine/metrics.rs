//! Tool metrics — Per Architecture §13.26 (Tool Metrics).
//! Re-exports metrics types from the telemetry module.

pub use crate::tool_engine::telemetry::{get_global_metrics, ToolMetricSummary, ToolMetrics};
