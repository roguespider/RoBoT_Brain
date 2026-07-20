// /src/experience/coordinator.rs

use crate::experience::{
    scorer::ExperienceScorer, types::*,
};

/// Events emitted by the experience system.
pub enum ExperienceEvent {
    Recorded(String),
    Scored(String),
    ReputationUpdated(String),
    ReflectionCompleted(String),
    HypothesisGenerated(String),
    ExplorationCompleted(String),
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
