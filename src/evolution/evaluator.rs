//! Evaluator — Per Architecture Chapter 26 (Controlled Experimentation).
//!
//! Evaluates experiment results to determine whether hypotheses are
//! supported or rejected, and whether improvement candidates should
//! be accepted.

/// Evaluates experiment results.
pub struct Evaluator;

impl Evaluator {
    /// Create a new evaluator.
    pub fn new() -> Self {
        Self
    }

    /// Evaluate whether experiment results support a hypothesis.
    pub fn evaluate(&self, success_rate: f32, threshold: f32) -> bool {
        success_rate >= threshold
    }

    /// Determine if a candidate should be promoted based on confidence.
    pub fn should_promote(&self, confidence: f32, threshold: f32) -> bool {
        confidence >= threshold
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
