//! Observability Events — Per Architecture Chapter 27 (Cognitive Monitoring).
//!
//! All major subsystems publish events. Events flow through an event bus
//! to observers for storage, visualization, and analysis.

/// A cognitive event emitted by a subsystem.
#[derive(Debug, Clone)]
pub struct CognitiveEvent {
    /// Unique event identifier.
    pub id: String,
    /// Timestamp of the event.
    pub timestamp: i64,
    /// Source subsystem that emitted the event.
    pub source_system: String,
    /// Type of event.
    pub event_type: String,
    /// Event payload data.
    pub payload: String,
    /// Confidence associated with the event.
    pub confidence: f32,
}

/// Categories of cognitive events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventCategory {
    /// Memory subsystem events.
    Memory,
    /// Experience subsystem events.
    Experience,
    /// Planning subsystem events.
    Planning,
    /// Execution subsystem events.
    Execution,
    /// Evolution subsystem events.
    Evolution,
}

/// An event with its category.
#[derive(Debug, Clone)]
pub struct CategorizedEvent {
    /// The event.
    pub event: CognitiveEvent,
    /// The category of the event.
    pub category: EventCategory,
}

impl CategorizedEvent {
    /// Create a new categorized event.
    pub fn new(event: CognitiveEvent, category: EventCategory) -> Self {
        Self { event, category }
    }
}
