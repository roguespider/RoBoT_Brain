// /src/experience/events/builders.rs

//! Event builders for creating ExperienceEvents
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts

use chrono::Utc;
use uuid::Uuid;

use super::{EventPayload, ExperienceEvent, ExperienceEventType};
use crate::experience::types::{Experience, ExperienceScore};

impl ExperienceEvent {
    /// Create an event indicating a new experience was recorded.
    pub(crate) fn recorded(experience_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExperienceRecorded,
            payload: EventPayload::Experience { experience_id },
        }
    }

    /// Create an event indicating a new experience was recorded with full experience data.
    pub fn experience_recorded(experience: Experience) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id: experience.id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExperienceRecorded,
            payload: EventPayload::ExperienceRecord {
                experience: Box::new(experience),
            },
        }
    }

    /// Create an event after an experience has been scored.
    pub fn scored(experience_id: Uuid, score: ExperienceScore) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::Scored,
            payload: EventPayload::Score {
                experience_id,
                score,
            },
        }
    }

    /// Create an event when reputation changes.
    pub fn reputation_updated(experience_id: Uuid, target_id: String, change: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReputationUpdated,
            payload: EventPayload::Reputation {
                entity_id: target_id,
                change,
            },
        }
    }

    /// Create an event when reflection completes.
    pub fn reflection_completed(experience_id: Uuid, reflection_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReflectionCompleted,
            payload: EventPayload::Reflection { reflection_id },
        }
    }

    /// Create an event when a hypothesis is generated.
    pub fn hypothesis_generated(experience_id: Uuid, hypothesis_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::HypothesisGenerated,
            payload: EventPayload::Hypothesis { hypothesis_id },
        }
    }

    /// Create an event when a hypothesis is validated.
    pub fn hypothesis_validated(
        experience_id: Uuid,
        hypothesis_id: String,
        validated: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::HypothesisValidated,
            payload: EventPayload::HypothesisValidation {
                hypothesis_id,
                result: if validated {
                    "supported".to_string()
                } else {
                    "rejected".to_string()
                },
            },
        }
    }

    /// Create an event when exploration starts.
    pub fn exploration_started(experience_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExplorationStarted,
            payload: EventPayload::Experience { experience_id },
        }
    }

    /// Create an event when exploration finishes.
    pub fn exploration_completed(experience_id: Uuid, exploration_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExplorationCompleted,
            payload: EventPayload::Exploration { exploration_id },
        }
    }

    /// Create an event when knowledge is updated.
    pub fn knowledge_updated(knowledge_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id: Uuid::nil(),
            timestamp: Utc::now(),
            event_type: ExperienceEventType::KnowledgeUpdated,
            payload: EventPayload::KnowledgeRecord { knowledge_id },
        }
    }

    /// Create an event when knowledge is transferred between domains.
    pub fn knowledge_transferred(
        experience_id: Uuid,
        source_domain: String,
        target_domain: String,
        count: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::KnowledgeTransferred,
            payload: EventPayload::KnowledgeTransfer {
                source_domain,
                target_domain,
                count,
            },
        }
    }
}
