// /src/experience/coordinator.rs

use anyhow::Result;

use crate::experience::{
    evolution::EvolutionEngine, exploration::ExplorationEngine, hypothesis::HypothesisEngine,
    recorder::ExperienceRecorder, reflection::ReflectionEngine, reputation::ReputationManager,
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
    EvolutionCompleted(String),
}

/// Coordinates the experience system.
///
/// The manager does not contain business logic.
/// Instead it orchestrates the specialized components.
pub struct ExperienceCoordinator {
    recorder: ExperienceRecorder,
    scorer: ExperienceScorer,
    reputation: ReputationManager,
    reflection: ReflectionEngine,
    exploration: ExplorationEngine,
    hypothesis: HypothesisEngine,
    evolution: EvolutionEngine,
}

impl ExperienceCoordinator {
    pub fn new(
        recorder: ExperienceRecorder,
        scorer: ExperienceScorer,
        reputation: ReputationManager,
        reflection: ReflectionEngine,
        exploration: ExplorationEngine,
        hypothesis: HypothesisEngine,
        evolution: EvolutionEngine,
    ) -> Self {
        Self {
            recorder,
            scorer,
            reputation,
            reflection,
            exploration,
            hypothesis,
            evolution,
        }
    }

    /// Process a completed experience through the learning pipeline.
    pub fn process(&self, mut experience: Experience) -> Result<Experience> {
        // Store the raw experience.
        self.recorder.record(&experience)?;

        // Score it.
        let score = self.scorer.score(&experience)?;
        experience.score = Some(score);

        // Update long-term reputation.
        self.reputation.update(&experience)?;

        // Notify learning systems.
        self.hypothesis.observe(&experience)?;
        self.exploration.observe(&experience)?;
        self.reflection.observe(&experience)?;
        self.evolution.observe(&experience)?;

        Ok(experience)
    }
}
