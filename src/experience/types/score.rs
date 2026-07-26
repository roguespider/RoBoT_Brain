// src/experience/types/score.rs
// Experience scoring types

use serde::{Deserialize, Serialize};

/// Score for an experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceScore {
    /// Overall importance of the experience.
    pub importance: f32,

    /// Confidence in the recorded outcome.
    pub confidence: f32,

    /// How different this experience is from previous ones.
    pub novelty: f32,

    /// Long-term reliability.
    pub reliability: f32,
}

impl Default for ExperienceScore {
    fn default() -> Self {
        Self {
            importance: 0.0,
            confidence: 0.0,
            novelty: 0.0,
            reliability: 0.0,
        }
    }
}
