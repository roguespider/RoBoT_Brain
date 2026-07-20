// robot/src/experience/hypothesis/core/lifecycle.rs

//! ============================================================================
//! HYPOTHESIS LIFECYCLE
//! ============================================================================
//!
//! Controls the progression of hypotheses through their lifetime.
//!
//! A hypothesis is not simply true or false. It moves through states as
//! evidence accumulates and confidence changes.
//!
//! This module owns state transitions only.
//! Confidence calculation belongs to the evaluator.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::hypothesis::{Hypothesis, HypothesisStatus};

/// ============================================================================
/// LIFECYCLE MANAGER
/// ============================================================================

#[derive(Debug, Clone)]
pub struct HypothesisLifecycle {
    /// Minimum evaluations before a hypothesis can become supported.
    pub minimum_evaluations: u32,

    /// Confidence required for support.
    pub supported_threshold: f32,

    /// Confidence below which a hypothesis is rejected.
    pub rejected_threshold: f32,
}

impl HypothesisLifecycle {
    pub fn new() -> Self {
        Self {
            minimum_evaluations: 5,

            supported_threshold: 0.85,

            rejected_threshold: 0.15,
        }
    }

    /// Evaluate whether a hypothesis should change state.
    pub fn update(&self, hypothesis: &mut Hypothesis) -> LifecycleResult {
        let previous = hypothesis.status;

        match hypothesis.status {
            HypothesisStatus::Draft => {
                if hypothesis.evaluations > 0 {
                    hypothesis.status = HypothesisStatus::Active;
                }
            }

            HypothesisStatus::Active => {
                if self.can_support(hypothesis) {
                    hypothesis.status = HypothesisStatus::Supported;
                } else if self.should_reject(hypothesis) {
                    hypothesis.status = HypothesisStatus::Rejected;
                }
            }

            HypothesisStatus::Supported => {
                if self.should_reject(hypothesis) {
                    hypothesis.status = HypothesisStatus::Rejected;
                }
            }

            HypothesisStatus::Rejected => {
                // Rejected hypotheses can still recover
                // if new evidence changes the picture.

                if self.can_support(hypothesis) {
                    hypothesis.status = HypothesisStatus::Supported;
                } else if hypothesis.evaluations > 0 && hypothesis.confidence.value > 0.30 {
                    hypothesis.status = HypothesisStatus::Active;
                }
            }

            HypothesisStatus::Archived => {
                // Archived hypotheses stay archived.
            }
        }

        if previous != hypothesis.status {
            hypothesis.touch();
        }

        LifecycleResult {
            changed: previous != hypothesis.status,

            previous,

            current: hypothesis.status,
        }
    }

    /// Manually archive a hypothesis.
    pub fn archive(&self, hypothesis: &mut Hypothesis) {
        hypothesis.status = HypothesisStatus::Archived;

        hypothesis.touch();
    }

    fn can_support(&self, hypothesis: &Hypothesis) -> bool {
        hypothesis.evaluations >= self.minimum_evaluations
            && hypothesis.confidence.value >= self.supported_threshold
    }

    fn should_reject(&self, hypothesis: &Hypothesis) -> bool {
        hypothesis.evaluations >= self.minimum_evaluations
            && hypothesis.confidence.value <= self.rejected_threshold
    }
}

impl Default for HypothesisLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// RESULT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleResult {
    pub changed: bool,

    pub previous: HypothesisStatus,

    pub current: HypothesisStatus,
}
