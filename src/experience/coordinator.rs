// /src/experience/coordinator.rs
// Experience system coordinator per Architecture §07


use crate::experience::{
    bus::ExperienceBus,
    events::ExperienceEvent,
    exploration::{
        Exploration, ExplorationAttempt, ExplorationFinding, Hypothesis,
        InMemoryExplorationRepository, ExplorationRepository,
    },
    scorer::ExperienceScorer, types::*,
};
use std::sync::Arc;
use uuid::Uuid;

/// Coordinates the experience system.
///
/// The manager does not contain business logic.
/// Instead it orchestrates the specialized components.
pub struct ExperienceCoordinator {
    scorer: ExperienceScorer,
    bus: Arc<ExperienceBus>,
    exploration_store: Arc<InMemoryExplorationRepository>,
}

impl ExperienceCoordinator {
    pub fn new(scorer: ExperienceScorer, bus: Arc<ExperienceBus>) -> Self {
        Self {
            scorer,
            bus,
            exploration_store: Arc::new(InMemoryExplorationRepository::new()),
        }
    }

    /// Process a completed experience through the learning pipeline.
    pub fn process(&self, mut experience: Experience) -> Experience {
        // Score it.
        let score = self.scorer.score(&experience);
        experience.score = Some(score.clone());

        // Publish scored event using builder
        let event = ExperienceEvent::scored(experience.id, score);
        let _ = self.bus.publish(event);

        experience
    }

    /// Record that an experience was created
    pub fn record_experience(&self, id: Uuid) {
        let event = ExperienceEvent::recorded(id);
        let _ = self.bus.publish(event);
    }

    /// Record that reflection was completed
    pub fn complete_reflection(&self, id: Uuid) {
        let reflection_id = Uuid::new_v4();
        let event = ExperienceEvent::reflection_completed(id, reflection_id);
        let _ = self.bus.publish(event);
    }

    /// Record that a hypothesis was generated
    pub fn generate_hypothesis(&self, id: Uuid) {
        let hypothesis_id = Uuid::new_v4();
        let event = ExperienceEvent::hypothesis_generated(id, hypothesis_id);
        let _ = self.bus.publish(event);
    }

    /// Record that exploration was completed
    pub fn complete_exploration(&self, id: Uuid) {
        let exploration_id = Uuid::new_v4();
        let event = ExperienceEvent::exploration_completed(id, exploration_id);
        let _ = self.bus.publish(event);
    }

    // === Exploration Management (wires Exploration lifecycle methods) ===

    /// Start a new exploration - uses Exploration::new() and Exploration::start()
    pub fn start_exploration(&self, id: String, title: String, purpose: String, context: ExperienceContext) -> Exploration {
        let mut exploration = Exploration::new(id, title, purpose, context);
        exploration.start();
        let _ = self.exploration_store.create(&exploration);
        let event = ExperienceEvent::exploration_completed(Uuid::new_v4(), Uuid::new_v4());
        let _ = self.bus.publish(event);
        exploration
    }

    /// Pause an exploration - uses Exploration::pause()
    pub fn pause_exploration(&self, id: &str) -> Option<Exploration> {
        let mut explorations = self.exploration_store.list_all().ok()?;
        if let Some(exp) = explorations.iter_mut().find(|e| e.id == id) {
            exp.pause();
            let _ = self.exploration_store.update(exp);
            return Some(exp.clone());
        }
        None
    }

    /// Complete an exploration - uses Exploration::complete()
    pub fn complete_exploration_with_result(&self, id: &str) -> Option<Exploration> {
        let mut explorations = self.exploration_store.list_all().ok()?;
        if let Some(exp) = explorations.iter_mut().find(|e| e.id == id) {
            exp.complete();
            let _ = self.exploration_store.update(exp);
            return Some(exp.clone());
        }
        None
    }

    /// Abandon an exploration - uses Exploration::abandon()
    pub fn abandon_exploration(&self, id: &str) -> Option<Exploration> {
        let mut explorations = self.exploration_store.list_all().ok()?;
        if let Some(exp) = explorations.iter_mut().find(|e| e.id == id) {
            exp.abandon();
            let _ = self.exploration_store.update(exp);
            return Some(exp.clone());
        }
        None
    }

    /// Add a hypothesis to an exploration - uses Exploration::add_hypothesis()
    pub fn add_hypothesis_to_exploration(&self, exploration_id: &str, hypothesis: Hypothesis) -> Option<()> {
        let mut explorations = self.exploration_store.list_all().ok()?;
        if let Some(exp) = explorations.iter_mut().find(|e| e.id == exploration_id) {
            exp.add_hypothesis(hypothesis);
            let _ = self.exploration_store.update(exp);
            return Some(());
        }
        None
    }

    /// Add an attempt to an exploration - uses Exploration::add_attempt()
    pub fn add_attempt_to_exploration(&self, exploration_id: &str, attempt: ExplorationAttempt) -> Option<()> {
        let mut explorations = self.exploration_store.list_all().ok()?;
        if let Some(exp) = explorations.iter_mut().find(|e| e.id == exploration_id) {
            exp.add_attempt(attempt);
            let _ = self.exploration_store.update(exp);
            return Some(());
        }
        None
    }

    /// Add a finding to an exploration - uses Exploration::add_finding()
    pub fn add_finding_to_exploration(&self, exploration_id: &str, finding: ExplorationFinding) -> Option<()> {
        let mut explorations = self.exploration_store.list_all().ok()?;
        if let Some(exp) = explorations.iter_mut().find(|e| e.id == exploration_id) {
            exp.add_finding(finding);
            let _ = self.exploration_store.update(exp);
            return Some(());
        }
        None
    }

    /// Get active explorations - uses Exploration::is_active()
    pub fn get_active_explorations(&self) -> Option<Vec<Exploration>> {
        let explorations = self.exploration_store.list_all().ok()?;
        Some(explorations.into_iter().filter(|e| e.is_active()).collect())
    }

    /// Check if an exploration is complete - uses Exploration::is_complete()
    pub fn is_exploration_complete(&self, id: &str) -> Option<bool> {
        let explorations = self.exploration_store.list_all().ok()?;
        Some(explorations.iter().find(|e| e.id == id)?.is_complete())
    }
}
