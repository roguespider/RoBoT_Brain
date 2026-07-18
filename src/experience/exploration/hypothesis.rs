use serde::{Deserialize, Serialize};

/// A belief or assumption being tested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,

    /// The assumption being tested
    pub statement: String,

    /// Confidence before testing
    pub confidence: f32,

    /// Result after investigation
    pub result: Option<HypothesisResult>,
}

/// Outcome of a hypothesis test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypothesisResult {
    Supported,
    PartiallySupported,
    Rejected,
    Unknown,
}
