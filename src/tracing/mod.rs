//! Observability — Cognitive monitoring and tracing (Architecture Chapter 27).
//!
//! Provides trace storage, event emission, and diagnostic visibility
//! without exposing internal subsystem state.
pub mod engine;
pub mod events;

pub use events::{CognitiveEvent, CognitiveEventType, EventSeverity, EventTracer};

/// Record a cognitive event to the persistent event store.
/// Per Architecture §27.10 (Trace Storage): events SHALL be persisted
/// to the cognitive_events table for trace replay and debugging.
pub fn record_cognitive_event(
    event_type: CognitiveEventType,
    correlation_id: &str,
    payload: serde_json::Value,
) -> Result<(), crate::tracing::ObservabilityError> {
    let payload_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(payload_error = %e, "Failed to serialize payload");
            return Err(crate::tracing::ObservabilityError::RecordingFailed);
        }
    };
    let payload_len = compute_payload_len(&payload);
    ::tracing::info!(
        event_type = ?event_type,
        correlation_id,
        payload_size = payload_len,
        "Persisting cognitive event"
    );
    // Persist to the cognitive_events table.
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for event recording");
            return Err(crate::tracing::ObservabilityError::RecordingFailed);
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for event recording");
            return Err(crate::tracing::ObservabilityError::RecordingFailed);
        }
    };
    let event_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();
    let event_type_name = event_type.name();
    let description = format!("Event type: {event_type_name}, correlation: {correlation_id}");
    if let Err(e) = conn.execute(
        "INSERT INTO cognitive_events (id, event_type, correlation_id, description, timestamp, payload) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            &event_id,
            event_type_name,
            correlation_id,
            &description,
            &timestamp.to_string(),
            &payload_str,
        ],
    ) {
        tracing::warn!(db_error = %e, "Failed to persist cognitive event");
        return Err(crate::tracing::ObservabilityError::RecordingFailed);
    }
    tracing::debug!(event_id, "Cognitive event persisted");
    Ok(())
}

/// Compute payload length for logging.
fn compute_payload_len(payload: &serde_json::Value) -> usize {
    match payload {
        serde_json::Value::String(s) => s.len(),
        serde_json::Value::Object(m) => m
            .values()
            .map(|v| match v {
                serde_json::Value::String(s) => s.len(),
                _ => v.to_string().len(),
            })
            .sum(),
        _ => 0,
    }
}

/// Replay events by correlation ID from the persistent event store.
/// Per Architecture §27.10 (Trace Storage): queries the cognitive_events table
/// and returns events ordered by timestamp.
pub fn replay_events(correlation_id: &str) -> Vec<TraceEvent> {
    if correlation_id.is_empty() {
        return Vec::new();
    }
    ::tracing::debug!(correlation_id, "Replaying events from DB");
    let db = match crate::database::sqlite::SqliteDatabase::initialize() {
        Ok(db) => db,
        Err(e) => {
            tracing::warn!(db_error = %e, "DB unavailable for trace replay");
            return Vec::new();
        }
    };
    let conn = match db.connection() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(db_error = %e, "No DB connection for trace replay");
            return Vec::new();
        }
    };
    let mut stmt = match conn.prepare(
        "SELECT id, event_type, description, timestamp, correlation_id \
         FROM cognitive_events WHERE correlation_id = ?1 \
         ORDER BY timestamp ASC",
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(db_error = %e, "Trace replay prepare failed");
            return Vec::new();
        }
    };
    let rows = match stmt.query_map([correlation_id], |row| {
        let id = row.get::<_, String>(0).unwrap_or_default();
        let event_type = row.get::<_, String>(1).unwrap_or_default();
        let description = row.get::<_, String>(2).unwrap_or_default();
        let timestamp = row.get::<_, i64>(3).unwrap_or(0);
        let correlation_id = row.get::<_, String>(4).unwrap_or_default();
        Ok(TraceEvent {
            id,
            event_type,
            timestamp,
            description,
            correlation_id,
        })
    }) {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect::<Vec<_>>(),
        Err(e) => {
            tracing::warn!(db_error = %e, "Trace replay query failed");
            Vec::new()
        }
    };
    if !rows.is_empty() {
        tracing::debug!(
            correlation_id,
            found = rows.len(),
            "Trace replay found events"
        );
    }
    rows
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
