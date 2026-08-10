//! Personality traits that influence behavior.

use serde::{Deserialize, Serialize};

/// Personality traits that influence behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityTraits {
    /// How curious the system is (0.0 - 1.0)
    pub curiosity: f32,

    /// How cautious the system is (0.0 - 1.0)
    pub caution: f32,

    /// How creative the system is (0.0 - 1.0)
    pub creativity: f32,

    /// How patient the system is (0.0 - 1.0)
    pub patience: f32,

    /// How thorough the system is (0.0 - 1.0)
    pub thoroughness: f32,

    /// Communication verbosity (0.0 = minimal, 1.0 = verbose)
    pub verbosity: f32,

    /// Risk tolerance (0.0 = risk-averse, 1.0 = risk-tolerant)
    pub risk_tolerance: f32,
}

impl Default for PersonalityTraits {
    fn default() -> Self {
        Self {
            curiosity: 0.7,
            caution: 0.5,
            creativity: 0.6,
            patience: 0.7,
            thoroughness: 0.8,
            verbosity: 0.5,
            risk_tolerance: 0.4,
        }
    }
}
