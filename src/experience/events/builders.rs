// /src/experience/events/builders.rs
//! Event builders for creating ExperienceEvents
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts



use chrono::Utc;
use uuid::Uuid;

use super::{EventPayload, ExperienceEvent, ExperienceEventType};
use crate::experience::types::{Experience, ExperienceScore};
use crate::experience::reflection::reflection::Reflection;
use crate::experience::hypothesis::core::hypothesis::Hypothesis;

impl ExperienceEvent {
    /// Create an event indicating a new experience was recorded.
    pub fn recorded(experience_id: Uuid) -> Self {
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
            payload: EventPayload::ExperienceRecord { experience },
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

    /// Create an event with full score data.
    pub fn score_recorded(experience_id: Uuid, score: ExperienceScore) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::Scored,
            payload: EventPayload::ScoreRecord {
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

    /// Create an event with full reputation data.
    pub fn reputation_changed(experience_id: Uuid, entity_id: String, previous: f32, current: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReputationUpdated,
            payload: EventPayload::ReputationRecord {
                entity_id,
                previous,
                current,
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

    /// Create an event with full reflection data.
    pub fn reflection_recorded(experience_id: Uuid, reflection: Reflection) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReflectionCompleted,
            payload: EventPayload::ReflectionRecord { reflection },
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

    /// Create an event with full hypothesis data.
    pub fn hypothesis_created(experience_id: Uuid, hypothesis: Hypothesis) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::HypothesisGenerated,
            payload: EventPayload::HypothesisRecord { hypothesis },
        }
    }

    /// Create an event when a hypothesis is validated.
    pub fn hypothesis_validated(experience_id: Uuid, hypothesis_id: String, validated: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::HypothesisValidated,
            payload: EventPayload::HypothesisValidation {
                hypothesis_id,
                result: if validated { "supported".to_string() } else { "rejected".to_string() },
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

    /// Create an event with full exploration data.
    pub fn exploration_finished(experience_id: Uuid, exploration_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExplorationCompleted,
            payload: EventPayload::ExplorationRecord { exploration_id },
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

    /// Create an event when evidence is added.
    pub fn evidence_added(experience_id: Uuid, evidence_id: Uuid, hypothesis_id: String, direction: String, strength: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::EvidenceAdded,
            payload: EventPayload::EvidenceRecord {
                evidence_id,
                hypothesis_id,
                direction,
                strength,
            },
        }
    }

    /// Create an event when a pattern is detected.
    pub fn pattern_detected(experience_id: Uuid, _pattern: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::PatternDetected,
            payload: EventPayload::Experience { experience_id },
        }
    }

    /// Create an event when a lesson is learned.
    pub fn lesson_learned(experience_id: Uuid, _lesson: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::LessonLearned,
            payload: EventPayload::Experience { experience_id },
        }
    }

    /// Create an event when confidence changes.
    pub fn confidence_changed(experience_id: Uuid, _previous: f32, _current: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ConfidenceChanged,
            payload: EventPayload::Experience { experience_id },
        }
    }

    /// Create an event when knowledge is promoted.
    pub fn knowledge_promoted(experience_id: Uuid, knowledge_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::KnowledgePromoted,
            payload: EventPayload::KnowledgeRecord { knowledge_id },
        }
    }

    /// Create an event when knowledge is deprecated.
    pub fn knowledge_deprecated(experience_id: Uuid, knowledge_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::KnowledgeDeprecated,
            payload: EventPayload::KnowledgeRecord { knowledge_id },
        }
    }

    /// Create an event when source trust changes.
    pub fn source_trust_changed(experience_id: Uuid, source: String, previous: f32, current: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            experience_id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::SourceTrustChanged,
            payload: EventPayload::ReputationRecord {
                entity_id: source,
                previous,
                current,
            },
        }
    }
}
