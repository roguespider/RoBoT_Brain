// /src/experience/events/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::payload::EventPayload;

/// ============================================================================
/// EXPERIENCE EVENT
/// ============================================================================

/// A signal emitted by the experience system.
///
/// Events are not memories themselves.
/// They notify subsystems that something happened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceEvent {
    /// Unique event identifier.
    pub id: Uuid,

    /// Experience this event belongs to.
    pub experience_id: Uuid,

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Category of event.
    pub event_type: ExperienceEventType,

    /// Data associated with the event.
    pub payload: EventPayload,
}

/// ============================================================================
/// EXPERIENCE EVENT TYPES
/// ============================================================================

/// Types of signals flowing through the experience system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperienceEventType {
    /// A new experience was recorded.
    Recorded,

    /// An experience was evaluated by the scorer.
    Scored,

    /// Reputation changed for a target.
    ReputationUpdated,

    /// Reflection completed.
    ReflectionCompleted,

    /// A new hypothesis was created.
    HypothesisGenerated,

    /// Exploration finished.
    ExplorationCompleted,

    /// Generic system event.
    System,

    /// Custom extension point.
    Custom(String),
}
