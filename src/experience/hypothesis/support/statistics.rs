// robot/src/experience/hypothesis/support/statistics.rs

//! ============================================================================
//! HYPOTHESIS STATISTICS
//! ============================================================================
//!
//! Tracks raw measurements for the hypothesis system.
//!
//! Statistics are simple counters and values.
//! Interpretation belongs to analytics.

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::hypothesis::{Hypothesis, HypothesisStatus};

/// ============================================================================
/// STATISTICS TRACKER
/// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypothesisStatistics {
    /// Total hypotheses observed.
    pub total_hypotheses: u64,

    /// Total evidence evaluations.
    pub total_evaluations: u64,

    /// Total supporting evidence events.
    pub total_confirmations: u64,

    /// Total contradiction events.
    pub total_contradictions: u64,

    /// Status counts.
    pub draft_count: u64,

    pub active_count: u64,

    pub supported_count: u64,

    pub rejected_count: u64,

    pub archived_count: u64,

    /// Confidence tracking.
    pub confidence_sum: f64,

    pub confidence_samples: u64,
}

impl HypothesisStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a hypothesis snapshot.
    pub fn record(&mut self, hypothesis: &Hypothesis) {
        self.total_hypotheses += 1;

        self.total_evaluations += hypothesis.evaluations as u64;

        self.total_confirmations += hypothesis.confirmations as u64;

        self.total_contradictions += hypothesis.contradictions as u64;

        match hypothesis.status {
            HypothesisStatus::Draft => {
                self.draft_count += 1;
            }

            HypothesisStatus::Active => {
                self.active_count += 1;
            }

            HypothesisStatus::Supported => {
                self.supported_count += 1;
            }

            HypothesisStatus::Rejected => {
                self.rejected_count += 1;
            }

            HypothesisStatus::Archived => {
                self.archived_count += 1;
            }
        }

        self.confidence_sum += hypothesis.confidence.value as f64;

        self.confidence_samples += 1;
    }

    /// Average confidence across recorded hypotheses.
    pub fn average_confidence(&self) -> f32 {
        if self.confidence_samples == 0 {
            return 0.0;
        }

        (self.confidence_sum / self.confidence_samples as f64) as f32
    }

    /// Percentage of hypotheses that became supported.
    pub fn support_rate(&self) -> f32 {
        if self.total_hypotheses == 0 {
            return 0.0;
        }

        self.supported_count as f32 / self.total_hypotheses as f32
    }

    /// Percentage of evaluations that supported beliefs.
    pub fn confirmation_rate(&self) -> f32 {
        if self.total_evaluations == 0 {
            return 0.0;
        }

        self.total_confirmations as f32 / self.total_evaluations as f32
    }

    /// Reset all counters.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// ============================================================================
/// SNAPSHOT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsSnapshot {
    pub total_hypotheses: u64,

    pub average_confidence: f32,

    pub support_rate: f32,

    pub confirmation_rate: f32,
}

impl From<&HypothesisStatistics> for StatisticsSnapshot {
    fn from(stats: &HypothesisStatistics) -> Self {
        Self {
            total_hypotheses: stats.total_hypotheses,

            average_confidence: stats.average_confidence(),

            support_rate: stats.clone().support_rate(),

            confirmation_rate: stats.clone().confirmation_rate(),
        }
    }
}
