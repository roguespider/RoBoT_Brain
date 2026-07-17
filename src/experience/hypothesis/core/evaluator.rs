//! ============================================================================
//! HYPOTHESIS EVALUATOR
//! ============================================================================
//!
//! Evaluates evidence against hypotheses.
//!
//! The evaluator does not create hypotheses and does not store them.
//! Its only responsibility is determining how evidence affects confidence.
//!
//! Future versions may incorporate:
//! - LLM reasoning
//! - pattern analysis
//! - Bayesian updates
//! - historical weighting
//! - reputation influence
//! - exploration results

use serde::{Deserialize, Serialize};

use super::{
    evidence::{Evidence, EvidenceRelationship},
    hypothesis::{Hypothesis, HypothesisStatus},
};

/// ============================================================================
/// EVALUATOR
/// ============================================================================

#[derive(Debug, Clone)]
pub struct HypothesisEvaluator {
    /// How strongly supporting evidence affects confidence.
    pub support_weight: f32,

    /// How strongly contradictory evidence affects confidence.
    pub contradiction_weight: f32,

    /// Minimum confidence before marking supported.
    pub supported_threshold: f32,

    /// Maximum confidence before considering rejected.
    pub rejected_threshold: f32,
}

impl HypothesisEvaluator {
    pub fn new() -> Self {
        Self {
            support_weight: 0.05,
            contradiction_weight: 0.08,

            supported_threshold: 0.85,
            rejected_threshold: 0.15,
        }
    }

    /// Evaluate evidence against a hypothesis.
    pub fn evaluate(&self, hypothesis: &mut Hypothesis, evidence: &Evidence) -> EvaluationResult {
        let previous_confidence = hypothesis.confidence.value;

        match evidence.relationship {
            EvidenceRelationship::Supports => {
                self.apply_support(hypothesis, evidence);
            }

            EvidenceRelationship::Contradicts => {
                self.apply_contradiction(hypothesis, evidence);
            }

            EvidenceRelationship::Neutral => {
                hypothesis.evaluations += 1;
            }
        }

        self.update_status(hypothesis);

        EvaluationResult {
            hypothesis_id: hypothesis.id.clone(),
            evidence_id: evidence.id.clone(),

            previous_confidence,

            new_confidence: hypothesis.confidence.value,

            relationship: evidence.relationship,

            changed: previous_confidence != hypothesis.confidence.value,
        }
    }

    fn apply_support(&self, hypothesis: &mut Hypothesis, evidence: &Evidence) {
        let adjustment = self.support_weight * evidence.weight();

        hypothesis.confidence.increase(adjustment);

        hypothesis.supporting_evidence.push(evidence.id.0.clone());

        hypothesis.confirmations += 1;
        hypothesis.evaluations += 1;

        hypothesis.touch();
    }

    fn apply_contradiction(&self, hypothesis: &mut Hypothesis, evidence: &Evidence) {
        let adjustment = self.contradiction_weight * evidence.weight();

        hypothesis.confidence.decrease(adjustment);

        hypothesis
            .contradicting_evidence
            .push(evidence.id.0.clone());

        hypothesis.contradictions += 1;
        hypothesis.evaluations += 1;

        hypothesis.touch();
    }

    fn update_status(&self, hypothesis: &mut Hypothesis) {
        let confidence = hypothesis.confidence.value;

        hypothesis.status = if hypothesis.evaluations < 3 {
            HypothesisStatus::Active
        } else if confidence >= self.supported_threshold {
            HypothesisStatus::Supported
        } else if confidence <= self.rejected_threshold {
            HypothesisStatus::Rejected
        } else {
            HypothesisStatus::Active
        };
    }
}

impl Default for HypothesisEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// ============================================================================
/// RESULT
/// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub hypothesis_id: super::hypothesis::HypothesisId,

    pub evidence_id: super::evidence::EvidenceId,

    pub previous_confidence: f32,

    pub new_confidence: f32,

    pub relationship: EvidenceRelationship,

    pub changed: bool,
}

