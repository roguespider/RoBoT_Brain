// /src/experience/events/payload.rs
//! Event payloads for all experience system events
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::experience::types::{Experience, ExperienceScore};
use crate::experience::reflection::reflection::Reflection;
use crate::experience::hypothesis::core::Hypothesis;

/// The specific event that occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    // -------------------------------------------------------------------------
    // Experience lifecycle (Per Architecture §5.3)
    // -------------------------------------------------------------------------
    /// A new experience was recorded.
    ExperienceRecord {
        /// The experience that was recorded
        experience: Experience,
    },

    /// An existing experience changed.
    ExperienceUpdated {
        experience_id: Uuid,
    },

    /// An experience was archived.
    ExperienceArchived {
        experience_id: Uuid,
    },

    /// An experience was deleted.
    ExperienceDeleted {
        experience_id: Uuid,
    },

    // -------------------------------------------------------------------------
    // Processing events (Per Architecture §5.3, §4.04)
    // -------------------------------------------------------------------------
    /// Scoring completed.
    ScoreRecord {
        experience_id: Uuid,
        score: ExperienceScore,
    },

    /// Reflection completed (Per Architecture §4.04).
    ReflectionRecord {
        reflection: Reflection,
    },

    /// Hypothesis generated (Per Architecture §4.04).
    HypothesisRecord {
        hypothesis: Hypothesis,
    },

    /// Hypothesis validated (Per Architecture §4.04).
    HypothesisValidation {
        hypothesis_id: String,
        result: String,
    },

    /// Knowledge updated (Per Architecture §4.04).
    KnowledgeRecord {
        knowledge_id: Uuid,
    },

    /// Reputation metrics changed (Per Architecture §4.04).
    ReputationRecord {
        entity_id: String,
        previous: f32,
        current: f32,
    },

    /// Exploration completed (Per Architecture §4.04).
    ExplorationRecord {
        exploration_id: Uuid,
    },

    // -------------------------------------------------------------------------
    // Evidence events (Per Architecture §11)
    // -------------------------------------------------------------------------
    /// Evidence added to a hypothesis
    EvidenceRecord {
        evidence_id: Uuid,
        hypothesis_id: String,
        direction: String, // "support" or "contradict"
        strength: f32,
    },

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
    // Legacy / misc (for backwards compatibility)
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

    /// Reflection has been requested.
    ReflectionRequested,

    /// Validation completed.
    ValidationCompleted { success: bool },

    /// Generic error associated with an experience.
    Error { message: String },

    // -------------------------------------------------------------------------
    // Builder payload variants (legacy compatibility)
    // -------------------------------------------------------------------------
    /// Generic event tied to an experience.
    Experience { experience_id: Uuid },

    /// Score tied to an experience.
    Score { experience_id: Uuid, score: ExperienceScore },

    /// Reputation change tied to an entity.
    Reputation { entity_id: String, change: f32 },

    /// Reflection tied to an ID.
    Reflection { reflection_id: Uuid },

    /// Hypothesis tied to an ID.
    Hypothesis { hypothesis_id: Uuid },

    /// Exploration tied to an ID.
    Exploration { exploration_id: Uuid },
}
