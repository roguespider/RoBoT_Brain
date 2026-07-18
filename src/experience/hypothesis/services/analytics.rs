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

use crate::experience::hypothesis::core::{Hypothesis, HypothesisStatus};

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
        let mut report = HypothesisAnalyticsReport::default();

        report.total = hypotheses.len() as u32;

        for hypothesis in hypotheses {
            match hypothesis.status {
                HypothesisStatus::Draft => {
                    report.draft += 1;
                }

                HypothesisStatus::Active => {
                    report.active += 1;
                }

                HypothesisStatus::Supported => {
                    report.supported += 1;
                }

                HypothesisStatus::Rejected => {
                    report.rejected += 1;
                }

                HypothesisStatus::Archived => {
                    report.archived += 1;
                }
            }

            report.total_confidence += hypothesis.confidence.value;

            report.total_evaluations += hypothesis.evaluations;
        }

        if report.total > 0 {
            report.average_confidence = report.total_confidence / report.total as f32;
        }

        report
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
