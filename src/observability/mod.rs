//! Observability — Cognitive monitoring and tracing (Architecture Chapter 27).
//!
//! Provides trace storage, event emission, and diagnostic visibility
//! without exposing internal subsystem state.
pub mod tracing;

/// Cognitive event types per Architecture Chapter 27.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CognitiveEventType {
    MemoryEvent,
    ExperienceEvent,
    PlanningEvent,
    ExecutionEvent,
    EvolutionEvent,
}

/// Record a cognitive event.
pub fn record_cognitive_event(
    event_type: CognitiveEventType,
    correlation_id: &str,
    payload: serde_json::Value,
) -> Result<(), crate::observability::ObservabilityError> {
    let payload_len = payload.get("key").map(|v| v.to_string().len()).unwrap_or(0);
    ::tracing::info!(
        event_type = ?event_type,
        correlation_id,
        payload_size = payload_len,
        "Cognitive event recorded"
    );
    Ok(())
}

/// Replay events by correlation ID.
pub fn replay_events(correlation_id: &str) -> Vec<TraceEvent> {
    // Placeholder: returns empty; real implementation would query event store
    ::tracing::debug!(correlation_id, "Replaying events");
    Vec::new()
}

/// A cognitive trace event.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEvent {
    /// Event identifier.
    pub id: String,
    /// Event type.
    pub event_type: String,
    /// Timestamp.
    pub timestamp: i64,
    /// Description.
    pub description: String,
    /// Correlation ID.
    pub correlation_id: String,
}

impl TraceEvent {
    /// Create a new trace event.
    pub fn new(event_type: &str, description: &str, correlation_id: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            description: description.to_string(),
            correlation_id: correlation_id.to_string(),
        }
    }
}

/// Observability error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservabilityError {
    RecordingFailed,
    ReplayFailed,
}

impl std::fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObservabilityError::RecordingFailed => write!(f, "recording failed"),
            ObservabilityError::ReplayFailed => write!(f, "replay failed"),
        }
    }
}

impl std::error::Error for ObservabilityError {}

/// Observability manager tracking cognitive events.
#[derive(Debug, Clone, Default)]
pub struct ObservabilityManager {
    /// Recorded events.
    events: Vec<TraceEvent>,
}

impl ObservabilityManager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Record an event.
    pub fn record_event(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    /// Get events by correlation ID.
    pub fn get_events_by_correlation(&self, correlation_id: &str) -> Vec<TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.correlation_id == correlation_id)
            .cloned()
            .collect()
    }

    /// Get all events.
    pub fn get_all_events(&self) -> Vec<TraceEvent> {
        self.events.clone()
    }
}
