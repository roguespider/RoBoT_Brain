//! Hypothesis — Per Architecture Chapter 26.7 (Hypothesis System).
//!
//! RoBoT uses hypotheses instead of assumptions. A hypothesis contains
//! a problem, proposed change, expected result, evidence, confidence,
//! testing method, and status.

/// A hypothesis about a potential improvement.
#[derive(Debug, Clone)]
pub struct Hypothesis {
    /// Unique hypothesis identifier.
    pub id: String,
    /// Problem being addressed.
    pub problem: String,
    /// Proposed change.
    pub proposed_change: String,
    /// Expected result.
    pub expected_result: String,
    /// Evidence supporting the hypothesis.
    pub evidence: Vec<String>,
    /// Confidence in the hypothesis (0.0-1.0).
    pub confidence: f32,
    /// Testing method.
    pub testing_method: String,
    /// Current evaluation status.
    pub status: HypothesisStatus,
}

/// Status of a hypothesis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypothesisStatus {
    /// Hypothesis has been created but not yet tested.
    Untested,
    /// Hypothesis is currently being tested.
    Testing,
    /// Hypothesis has been supported by evidence.
    Supported,
    /// Hypothesis has been refuted.
    Rejected,
    /// Hypothesis needs more evidence.
    Inconclusive,
}

impl Hypothesis {
    /// Create a new hypothesis.
    pub fn new(
        id: String,
        problem: String,
        proposed_change: String,
        expected_result: String,
        testing_method: String,
    ) -> Self {
        Self {
            id,
            problem,
            proposed_change,
            expected_result,
            evidence: Vec::new(),
            confidence: 0.5,
            testing_method,
            status: HypothesisStatus::Untested,
        }
    }

    /// Add evidence to the hypothesis.
    pub fn add_evidence(&mut self, evidence: String) {
        self.evidence.push(evidence);
    }

    /// Update confidence.
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// Update status.
    pub fn set_status(&mut self, status: HypothesisStatus) {
        self.status = status;
    }
}
