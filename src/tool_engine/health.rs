//! Tool health monitoring — Per Architecture §13.25 (Tool Health Monitoring).
//! Re-exports health types from the telemetry module.

pub use crate::tool_engine::telemetry::{HealthThresholds, ToolHealthMonitor, ToolHealthStatus};
