// /src/experience/coordinator.rs

use crate::experience::{
    bus::ExperienceBus,
    events::{types::{ExperienceEvent, ExperienceEventType}, payload::EventPayload},
    scorer::ExperienceScorer, types::*,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Coordinates the experience system.
///
/// The manager does not contain business logic.
/// Instead it orchestrates the specialized components.
pub struct ExperienceCoordinator {
    scorer: ExperienceScorer,
    bus: Arc<ExperienceBus>,
}

impl ExperienceCoordinator {
    pub fn new(scorer: ExperienceScorer, bus: Arc<ExperienceBus>) -> Self {
        Self { scorer, bus }
    }

    /// Process a completed experience through the learning pipeline.
    pub fn process(&self, mut experience: Experience) -> Experience {
        // Score it.
        let score = self.scorer.score(&experience);
        experience.score = Some(score);

        // Publish scored event
        let event = ExperienceEvent {
            id: Uuid::new_v4(),
            experience_id: experience.id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::Scored,
            payload: EventPayload::Experience { experience_id: experience.id },
        };
        let _ = self.bus.publish(event);

        experience
    }

    /// Record that an experience was created
    pub fn record_experience(&self, id: Uuid) {
        let event = ExperienceEvent {
            id: Uuid::new_v4(),
            experience_id: id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExperienceRecorded,
            payload: EventPayload::Experience { experience_id: id },
        };
        let _ = self.bus.publish(event);
    }

    /// Record that reflection was completed
    pub fn complete_reflection(&self, id: Uuid) {
        let event = ExperienceEvent {
            id: Uuid::new_v4(),
            experience_id: id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ReflectionCompleted,
            payload: EventPayload::Reflection { reflection_id: Uuid::new_v4() },
        };
        let _ = self.bus.publish(event);
    }

    /// Record that a hypothesis was generated
    pub fn generate_hypothesis(&self, id: Uuid) {
        let event = ExperienceEvent {
            id: Uuid::new_v4(),
            experience_id: id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::HypothesisGenerated,
            payload: EventPayload::Hypothesis { hypothesis_id: Uuid::new_v4() },
        };
        let _ = self.bus.publish(event);
    }

    /// Record that exploration was completed
    pub fn complete_exploration(&self, id: Uuid) {
        let event = ExperienceEvent {
            id: Uuid::new_v4(),
            experience_id: id,
            timestamp: Utc::now(),
            event_type: ExperienceEventType::ExplorationCompleted,
            payload: EventPayload::Exploration { exploration_id: Uuid::new_v4() },
        };
        let _ = self.bus.publish(event);
    }
}
