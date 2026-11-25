//! Evolution Manager — coordinates controlled self-improvement.
//!
//! Per Architecture Chapter 26: tracks improvement candidates, manages
//! experiments, evaluates results, and records changes.

use crate::evolution::{CandidateStatus, Experiment, Hypothesis, ImprovementCandidate};

/// Evolution Manager — coordinates controlled self-improvement.
pub struct EvolutionManager {
    /// Pending improvement candidates.
    candidates: Vec<ImprovementCandidate>,
    /// Active hypotheses.
    hypotheses: Vec<Hypothesis>,
    /// Active experiments.
    experiments: Vec<Experiment>,
}

impl EvolutionManager {
    /// Create a new evolution manager.
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            hypotheses: Vec::new(),
            experiments: Vec::new(),
        }
    }

    /// Register a new improvement candidate.
    pub fn register_candidate(&mut self, candidate: ImprovementCandidate) {
        self.candidates.push(candidate);
    }

    /// Register a new hypothesis.
    pub fn register_hypothesis(&mut self, hypothesis: Hypothesis) {
        self.hypotheses.push(hypothesis);
    }

    /// Register a new experiment.
    pub fn register_experiment(&mut self, experiment: Experiment) {
        self.experiments.push(experiment);
    }

    /// Get all candidates.
    pub fn candidates(&self) -> &[ImprovementCandidate] {
        &self.candidates
    }

    /// Get all hypotheses.
    pub fn hypotheses(&self) -> &[Hypothesis] {
        &self.hypotheses
    }

    /// Get all experiments.
    pub fn experiments(&self) -> &[Experiment] {
        &self.experiments
    }

    /// Count pending candidates.
    pub fn pending_candidate_count(&self) -> usize {
        self.candidates
            .iter()
            .filter(|c| matches!(c.status, CandidateStatus::Proposed))
            .count()
    }
}

impl Default for EvolutionManager {
    fn default() -> Self {
        Self::new()
    }
}
