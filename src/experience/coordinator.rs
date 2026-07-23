// /src/experience/coordinator.rs

use crate::experience::{
    events::types::ExperienceEventType,
    scorer::ExperienceScorer, types::*,
};

/// Events emitted by the experience system.
#[derive(Debug, Clone)]
pub enum ExperienceEvent {
    Recorded(String),
    Scored(String),
    ReputationUpdated(String),
    ReflectionCompleted(String),
    HypothesisGenerated(String),
    ExplorationCompleted(String),
}

impl ExperienceEvent {
    pub fn event_type(&self) -> ExperienceEventType {
        match self {
            ExperienceEvent::Recorded(_) => ExperienceEventType::ExperienceRecorded,
            ExperienceEvent::Scored(_) => ExperienceEventType::Scored,
            ExperienceEvent::ReputationUpdated(_) => ExperienceEventType::ReputationUpdated,
            ExperienceEvent::ReflectionCompleted(_) => ExperienceEventType::ReflectionCompleted,
            ExperienceEvent::HypothesisGenerated(_) => ExperienceEventType::HypothesisGenerated,
            ExperienceEvent::ExplorationCompleted(_) => ExperienceEventType::ExplorationCompleted,
        }
    }
}

/// Coordinates the experience system.
///
/// The manager does not contain business logic.
/// Instead it orchestrates the specialized components.
pub struct ExperienceCoordinator {
    scorer: ExperienceScorer,
}

impl ExperienceCoordinator {
    pub fn new(scorer: ExperienceScorer) -> Self {
        Self { scorer }
    }

    /// Process a completed experience through the learning pipeline.
    pub fn process(&self, mut experience: Experience) -> Experience {
        // Score it.
        let score = self.scorer.score(&experience);
        experience.score = Some(score);

        experience
    }
}
