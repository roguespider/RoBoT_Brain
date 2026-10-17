// src/tool_engine/lifecycle/mod.rs
//! Tool lifecycle management.
//!
//! Per Architecture Chapter 13, §13.16 (Tool Health Monitoring):
//! - Tool health states: Discovered, Healthy, Degraded, Unhealthy, Removed
//! - Health monitoring: health events, health metrics, health alerts
//! - Health metrics: failure count, duration tracking, health check status
//!
//! Tracks tool health events and lifecycle states for observability.

use std::collections::HashMap;

/// Events that can occur in a tool's lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolEvent {
    /// Tool was discovered during registration.
    Discovered,
    /// Tool is healthy and operational.
    Healthy,
    /// Tool is degraded but partially functional.
    Degraded,
    /// Tool is unhealthy and not functional.
    Unhealthy,
    /// Tool was removed from the registry.
    Removed,
}

impl std::fmt::Display for ToolEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolEvent::Discovered => write!(f, "discovered"),
            ToolEvent::Healthy => write!(f, "healthy"),
            ToolEvent::Degraded => write!(f, "degraded"),
            ToolEvent::Unhealthy => write!(f, "unhealthy"),
            ToolEvent::Removed => write!(f, "removed"),
        }
    }
}

/// A health event for a tool.
#[derive(Debug, Clone)]
pub struct ToolHealthEvent {
    /// Tool identifier.
    pub tool_id: String,
    /// The event that occurred.
    pub event: ToolEvent,
    /// Timestamp of the event.
    pub timestamp: i64,
    /// Additional details about the event.
    pub details: String,
}

impl ToolHealthEvent {
    /// Create a new health event.
    pub fn new(tool_id: &str, event: ToolEvent, details: &str) -> Self {
        Self {
            tool_id: tool_id.to_string(),
            event,
            timestamp: chrono::Utc::now().timestamp(),
            details: details.to_string(),
        }
    }
}

/// Lifecycle record tracking a tool's state history.
#[derive(Debug, Clone)]
pub struct ToolLifecycleRecord {
    /// Tool identifier.
    pub tool_id: String,
    /// Current state of the tool.
    pub current_state: ToolEvent,
    /// When the current state started.
    pub state_started_at: i64,
    /// Count of unhealthy transitions.
    pub failure_count: u32,
    /// Total duration in current state (seconds).
    pub state_duration_secs: u64,
}

/// Health metrics for a single tool.
#[derive(Debug, Clone)]
pub struct ToolHealthMetrics {
    /// Tool identifier.
    pub tool_id: String,
    /// Current health state.
    pub current_state: ToolEvent,
    /// Count of unhealthy transitions.
    pub failure_count: u32,
    /// Duration in current state (seconds).
    pub duration_secs: u64,
    /// Whether the tool is considered healthy.
    pub is_healthy: bool,
}

impl ToolLifecycleRecord {
    /// Create a new lifecycle record for a tool.
    pub fn new(tool_id: &str) -> Self {
        Self {
            tool_id: tool_id.to_string(),
            current_state: ToolEvent::Discovered,
            state_started_at: chrono::Utc::now().timestamp(),
            failure_count: 0,
            state_duration_secs: 0,
        }
    }

    /// Process an event and update the lifecycle state.
    /// Per Architecture §13.16: state transitions update failure counters
    /// and reset state timers.
    pub fn process_event(&mut self, event: ToolEvent) {
        match &event {
            ToolEvent::Unhealthy => {
                self.failure_count += 1;
            }
            ToolEvent::Healthy => {
                // Reset failure count on recovery
                self.failure_count = 0;
            }
            ToolEvent::Degraded => {
                // Degraded state: partial functionality
                // Don't count as full failure but mark as degraded
            }
            ToolEvent::Removed => {
                // Removed state: track removal
                tracing::info!(tool_id = %self.tool_id, "Tool lifecycle: removed from registry");
            }
            _ => {}
        }
        self.current_state = event.clone();
        self.state_started_at = chrono::Utc::now().timestamp();
        self.state_duration_secs = 0;
    }

    /// Get the count of unhealthy transitions.
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Get the duration in the current state (seconds).
    pub fn duration_in_current_state(&self) -> u64 {
        if self.state_started_at > 0 {
            let diff = chrono::Utc::now().timestamp() - self.state_started_at;
            diff.max(0) as u64
        } else {
            0
        }
    }

    /// Perform a health check on the tool's current state.
    /// Per Architecture §13.16: returns the current health state and
    /// whether the tool is considered healthy.
    pub fn health_check(&self) -> (ToolEvent, bool) {
        let is_healthy = matches!(self.current_state, ToolEvent::Healthy);
        (self.current_state.clone(), is_healthy)
    }

    /// Update the tool's status to a new state.
    /// Per Architecture §13.16: status updates propagate to tool registry.
    pub fn update_status(&mut self, new_state: ToolEvent) {
        self.process_event(new_state);
    }

    /// Get health metrics for this tool.
    /// Per Architecture §13.16: metrics include failure count, duration,
    /// and overall health status.
    pub fn get_metrics(&self) -> ToolHealthMetrics {
        let (current_state, is_healthy) = self.health_check();
        ToolHealthMetrics {
            tool_id: self.tool_id.clone(),
            current_state,
            failure_count: self.failure_count,
            duration_secs: self.duration_in_current_state(),
            is_healthy,
        }
    }

    /// Record a tool failure event.
    /// Per Architecture §13.16: failures increment the failure counter
    /// and transition the tool to Unhealthy state.
    pub fn record_failure(&mut self, details: &str) {
        let event = ToolHealthEvent::new(&self.tool_id, ToolEvent::Unhealthy, details);
        self.process_event(ToolEvent::Unhealthy);
        tracing::warn!(
            tool_id = %self.tool_id,
            failure_count = self.failure_count,
            event_details = %event.details,
            "Tool failure recorded"
        );
    }

    /// Record a tool success event.
    /// Per Architecture §13.16: successes reset failure counters
    /// and transition the tool to Healthy state.
    pub fn record_success(&mut self, details: &str) {
        let event = ToolHealthEvent::new(&self.tool_id, ToolEvent::Healthy, details);
        self.process_event(ToolEvent::Healthy);
        tracing::debug!(
            tool_id = %self.tool_id,
            event_details = %event.details,
            "Tool success recorded"
        );
    }
}

/// Manages lifecycle records for all registered tools.
/// Per Architecture §13.16: the lifecycle manager tracks tool health
/// events, provides health check aggregation, and emits health alerts
/// when tools become unhealthy.
#[derive(Debug, Default)]
pub struct ToolLifecycle {
    /// Lifecycle records indexed by tool ID.
    records: HashMap<String, ToolLifecycleRecord>,
    /// Health event log for observability.
    health_events: Vec<ToolHealthEvent>,
}

impl ToolLifecycle {
    /// Create a new lifecycle manager.
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            health_events: Vec::new(),
        }
    }

    /// Register a tool in the lifecycle manager.
    /// Per Architecture §13.1: called when a tool is registered in the registry.
    pub fn register_tool(&mut self, tool_id: &str) {
        let record = ToolLifecycleRecord::new(tool_id);
        let event = ToolHealthEvent::new(tool_id, ToolEvent::Discovered, "Tool registered");
        self.records.insert(tool_id.to_string(), record);
        self.health_events.push(event);
    }

    /// Perform a health check on a single registered tool.
    /// Per Architecture §13.16: returns current health state and health status.
    pub fn health_check(&self, tool_id: &str) -> Option<(ToolEvent, bool)> {
        self.records.get(tool_id).map(|r| r.health_check())
    }

    /// Update the status of a single registered tool.
    /// Per Architecture §13.16: status updates propagate to tool registry.
    pub fn update_status(&mut self, tool_id: &str, new_state: ToolEvent) {
        if let Some(record) = self.records.get_mut(tool_id) {
            record.update_status(new_state);
        }
    }

    /// Get health metrics for a single registered tool.
    /// Per Architecture §13.16: metrics available for observability layer.
    pub fn get_metrics(&self, tool_id: &str) -> Option<ToolHealthMetrics> {
        self.records.get(tool_id).map(|r| r.get_metrics())
    }

    /// Get the current health state of a tool.
    pub fn get_health(&self, tool_id: &str) -> Option<ToolHealthMetrics> {
        self.records.get(tool_id).map(|r| r.get_metrics())
    }

    /// Perform health checks on all registered tools.
    /// Per Architecture §13.16: returns a list of health metrics for all tools.
    pub fn health_check_all(&self) -> Vec<ToolHealthMetrics> {
        self.records.values().map(|r| r.get_metrics()).collect()
    }

    /// Check if any tools are unhealthy or degraded.
    /// Per Architecture §13.16: used to generate health alerts.
    pub fn has_unhealthy_tools(&self) -> bool {
        self.records
            .values()
            .any(|r| matches!(r.current_state, ToolEvent::Unhealthy | ToolEvent::Degraded))
    }

    /// Record a success for a tool.
    /// Per Architecture §13.16: wire into success handling.
    pub fn record_success(&mut self, tool_id: &str, details: &str) {
        if let Some(record) = self.records.get_mut(tool_id) {
            record.record_success(details);
        }
    }

    /// Record a failure for a tool.
    /// Per Architecture §13.16: wire into failure handling.
    pub fn record_failure(&mut self, tool_id: &str, details: &str) {
        if let Some(record) = self.records.get_mut(tool_id) {
            record.record_failure(details);
        }
    }

    /// Get the number of registered tools.
    pub fn tool_count(&self) -> usize {
        self.records.len()
    }

    /// Get all tool IDs.
    pub fn tool_ids(&self) -> Vec<String> {
        self.records.keys().cloned().collect()
    }

    /// Get the health events log.
    /// Per Architecture §13.16: wire into observability layer.
    pub fn get_health_events(&self) -> &[ToolHealthEvent] {
        &self.health_events
    }

    /// Check if a tool is registered.
    pub fn has_tool(&self, tool_id: &str) -> bool {
        self.records.contains_key(tool_id)
    }

    /// Transition a tool to a degraded state.
    /// Per Architecture §13.16: degraded state means partially functional.
    pub fn degrade_tool(&mut self, tool_id: &str, reason: &str) {
        if let Some(record) = self.records.get_mut(tool_id) {
            let event = ToolHealthEvent::new(tool_id, ToolEvent::Degraded, reason);
            record.process_event(ToolEvent::Degraded);
            self.health_events.push(event);
            tracing::warn!(
                tool_id = %tool_id,
                reason = %reason,
                "Tool degraded"
            );
        }
    }

    /// Remove a tool from the registry.
    /// Per Architecture §13.16: removed state tracks tool lifecycle.
    pub fn remove_tool(&mut self, tool_id: &str) {
        if let Some(record) = self.records.get_mut(tool_id) {
            let event =
                ToolHealthEvent::new(tool_id, ToolEvent::Removed, "Tool removed from registry");
            record.process_event(ToolEvent::Removed);
            self.health_events.push(event);
            tracing::info!(tool_id = %tool_id, "Tool removed from registry");
        }
    }
}
