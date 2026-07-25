//! Hypothesis - a belief or assumption being tested through exploration.
//!
//! Per Architecture §2.7, hypotheses enable discovery by allowing RoBoT to
//! propose explanations, test them through exploration, and update confidence.

use serde::{Deserialize, Serialize};

/// A belief or assumption being tested through exploration.
///
/// Hypotheses represent possible explanations that have not yet become knowledge.
/// They are assigned confidence and tested through exploration attempts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    /// Unique hypothesis identifier
    pub id: String,

    /// The assumption or explanation being tested
    pub statement: String,

    /// Confidence level before testing (0.0 - 1.0)
    pub confidence: f32,

    /// Result after investigation
    pub result: Option<HypothesisResult>,
}

impl Hypothesis {
    /// Create a new hypothesis with the given statement.
    pub fn new(id: String, statement: String, initial_confidence: f32) -> Self {
        Self {
            id,
            statement,
            confidence: initial_confidence.clamp(0.0, 1.0),
            result: None,
        }
    }

    /// Set the result of testing this hypothesis.
    pub fn set_result(&mut self, result: HypothesisResult) {
        self.result = Some(result);
    }

    /// Update confidence based on test results.
    pub fn update_confidence(&mut self, new_confidence: f32) {
        self.confidence = new_confidence.clamp(0.0, 1.0);
    }
}

/// Outcome of testing a hypothesis.
///
/// Per Architecture §11 (Hypothesis and Reasoning), hypotheses are tested
/// and assigned results based on evidence gathered during exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypothesisResult {
    /// Evidence fully supports this hypothesis
    Supported,
    /// Evidence partially supports this hypothesis
    PartiallySupported,
    /// Evidence contradicts this hypothesis
    Rejected,
    /// Unable to determine result from evidence
    Unknown,
}
