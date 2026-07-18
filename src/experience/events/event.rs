// /src/experience/events/event.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::payload::EventPayload;

/// A single immutable event flowing through the Experience system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceEvent {
    /// Unique ID for this event.
    pub id: Uuid,

    /// Experience this event belongs to.
    pub experience_id: Uuid,

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,

    /// What happened.
    pub payload: EventPayload,
}

impl ExperienceEvent {
    /// Create a new event with an automatically generated ID and timestamp.
    pub fn new(experience_id: Uuid, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            payload,
        }
    }
}
