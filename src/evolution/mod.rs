//! Evolution Engine — Per Architecture Chapter 26 (Self-Improvement and Evolution).
//!
//! Implements controlled self-improvement through experience analysis,
//! hypothesis testing, experimentation, and validated capability evolution.

pub mod candidates;
pub mod evaluator;
pub mod experiment;
pub mod history;
pub mod hypothesis;
pub mod manager;

pub use candidates::{CandidateStatus, ImprovementCandidate};
pub use evaluator::Evaluator;
pub use experiment::Experiment;
pub use history::{EvolutionHistory, EvolutionRecord};
pub use hypothesis::{Hypothesis, HypothesisStatus};
pub use manager::EvolutionManager;
