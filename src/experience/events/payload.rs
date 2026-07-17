// /src/experience/events/payload.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The specific event that occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    // -------------------------------------------------------------------------
    // Experience lifecycle
    // -------------------------------------------------------------------------
    /// A new experience was recorded.
    ExperienceRecorded,

    /// An existing experience changed.
    ExperienceUpdated,

    /// An experience was archived.
    ExperienceArchived,

    /// An experience was deleted.
    ExperienceDeleted,

    // -------------------------------------------------------------------------
    // Processing results
    // -------------------------------------------------------------------------
    /// Scoring completed.
    ScoreCalculated { score: f32 },

    /// Reputation metrics changed.
    ReputationUpdated { previous: f32, current: f32 },

    /// Reflection completed.
    ReflectionCompleted { reflection_id: Uuid },

    /// Hypothesis generated.
    HypothesisGenerated { hypothesis_id: Uuid },

    /// Exploration completed.
    ExplorationCompleted { exploration_id: Uuid },

    // -------------------------------------------------------------------------
    // Observer lifecycle
    // -------------------------------------------------------------------------
    /// An observer started.
    ObserverStarted { observer: String },

    /// An observer shut down normally.
    ObserverStopped { observer: String },

    /// An observer encountered a fatal error.
    ObserverFailed { observer: String, error: String },

    // -------------------------------------------------------------------------
    // Processing failures
    // -------------------------------------------------------------------------
    /// A processing stage failed but the observer remained healthy.
    ProcessingFailed { stage: String, error: String },

    // -------------------------------------------------------------------------
    // Legacy / misc
    // -------------------------------------------------------------------------
    /// Reflection has been requested.
    ReflectionRequested,

    /// Validation completed.
    ValidationCompleted { success: bool },

    /// Generic error associated with an experience.
    Error { message: String },

    // -------------------------------------------------------------------------
    // Builders payload variants
    // -------------------------------------------------------------------------
    /// Generic event tied to an experience.
    Experience { experience_id: Uuid },

    /// Score tied to an experience.
    Score { experience_id: Uuid, score: f32 },

    /// Reputation change tied to an entity.
    Reputation { entity_id: String, change: f32 },

    /// Reflection tied to an ID.
    Reflection { reflection_id: Uuid },

    /// Hypothesis tied to an ID.
    Hypothesis { hypothesis_id: Uuid },

    /// Exploration tied to an ID.
    Exploration { exploration_id: Uuid },
}
