// /src/experience/reputation/factors.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReputationFactor {
    Accuracy,

    Reasoning,

    Coding,

    Creativity,

    Reliability,

    Speed,

    Safety,

    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorScore {
    pub factor: ReputationFactor,

    pub score: f64,

    pub observations: u64,
}

impl FactorScore {
    pub fn new(factor: ReputationFactor) -> Self {
        Self {
            factor,
            score: 0.5,
            observations: 0,
        }
    }
}
