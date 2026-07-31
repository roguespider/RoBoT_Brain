// robot/src/experience/hypothesis/services/analytics.rs



//! ============================================================================
//! HYPOTHESIS ANALYTICS
//! ============================================================================
//!
//! Provides analysis and reporting for the hypothesis system.
//!
//! Analytics does not modify hypotheses.
//! It observes the current state and produces metrics.

use serde::{Deserialize, Serialize};

use crate::experience::hypothesis::core::hypothesis::{Hypothesis, HypothesisStatus};

/// ============================================================================
/// ANALYTICS SERVICE
/// ============================================================================
#[derive(Debug, Clone, Default)]
pub struct HypothesisAnalytics;

impl HypothesisAnalytics {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a collection of hypotheses.
    pub fn analyze(&self, hypotheses: &[Hypothesis]) -> HypothesisAnalyticsReport {
        let total = hypotheses.len() as u32;
        let mut draft = 0u32;
        let mut active = 0u32;
        let mut supported = 0u32;
        let mut rejected = 0u32;
        let mut archived = 0u32;
        let mut total_confidence = 0.0f32;
        let mut total_evaluations = 0u32;

        for hypothesis in hypotheses {
            match hypothesis.status {
                HypothesisStatus::Draft => {
                    draft += 1;
                }

                HypothesisStatus::Active => {
                    active += 1;
                }

                HypothesisStatus::Supported => {
                    supported += 1;
                }

                HypothesisStatus::Rejected => {
                    rejected += 1;
                }

                HypothesisStatus::Archived => {
                    archived += 1;
                }
            }

            total_confidence += hypothesis.confidence.value;
            total_evaluations += hypothesis.evaluations;
        }

        let average_confidence = if total > 0 {
            total_confidence / total as f32
        } else {
            0.0
        };

        HypothesisAnalyticsReport {
            total,
            draft,
            active,
            supported,
            rejected,
            archived,
            average_confidence,
            total_confidence,
            total_evaluations,
        }
    }

    /// Determine whether the hypothesis system is stable.
    ///
    /// A stable system has many evaluated hypotheses and
    /// fewer rejected beliefs.
    pub fn stability_score(&self, report: &HypothesisAnalyticsReport) -> f32 {
        if report.total == 0 {
            return 0.0;
        }

        let accepted = report.supported + report.active;

        accepted as f32 / report.total as f32
    }
}

/// ============================================================================
/// REPORT
/// ============================================================================
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HypothesisAnalyticsReport {
    pub total: u32,

    pub draft: u32,

    pub active: u32,

    pub supported: u32,

    pub rejected: u32,

    pub archived: u32,

    pub average_confidence: f32,

    pub total_confidence: f32,

    pub total_evaluations: u32,
}
