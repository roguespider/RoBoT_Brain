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

impl PersonalityTraits {
    /// Weight an action's base score by personality traits.
    /// Adjusts by risk_tolerance (increases score for risk-tolerant) and caution (decreases score for cautious).
    pub fn weight_action(&self, base_score: f32) -> f32 {
        let risk_factor = 1.0 + (self.risk_tolerance - 0.5) * 0.4; // -0.2 to +0.2 range
        let caution_factor = 1.0 - self.caution * 0.2; // -0.0 to -0.2 range
        (base_score * risk_factor * caution_factor).clamp(0.0, 1.0)
    }

    /// Decide whether to act based on confidence and traits.
    /// Cautious agents require confidence > 0.7; bold agents require > 0.3.
    pub fn should_act(&self, confidence: f32) -> bool {
        let threshold = if self.caution > 0.6 && self.risk_tolerance < 0.3 {
            // Cautious: high bar
            0.7
        } else if self.risk_tolerance > 0.7 && self.caution < 0.3 {
            // Bold: low bar
            0.3
        } else {
            // Base threshold
            0.5
        };
        confidence >= threshold
    }

    /// Adapt traits based on feedback (0.0 = negative, 1.0 = positive).
    pub fn adapt_traits(&mut self, feedback: f32) {
        let adjustment = (feedback - 0.5) * 0.05; // ±0.025 range
        self.curiosity = (self.curiosity + adjustment * 0.3).clamp(0.0, 1.0);
        self.caution = (self.caution - adjustment * 0.2).clamp(0.0, 1.0);
        self.creativity = (self.creativity + adjustment * 0.5).clamp(0.0, 1.0);
        self.patience = (self.patience + adjustment * 0.1).clamp(0.0, 1.0);
        self.thoroughness = (self.thoroughness + adjustment * 0.15).clamp(0.0, 1.0);
        self.verbosity = (self.verbosity + adjustment * 0.1).clamp(0.0, 1.0);
        self.risk_tolerance = (self.risk_tolerance + adjustment * 0.4).clamp(0.0, 1.0);
    }
}
