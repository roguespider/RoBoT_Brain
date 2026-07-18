// /src/experience/reputation/reputation.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::factors::{FactorScore, ReputationFactor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    /// Entity being evaluated.
    pub id: String,

    /// Overall reputation.
    pub score: f64,

    /// Individual trust dimensions.
    pub factors: Vec<FactorScore>,

    pub observations: u64,

    pub successes: u64,

    pub failures: u64,

    pub updated_at: DateTime<Utc>,

    pub history: Vec<ReputationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: DateTime<Utc>,

    /// Related experience.
    pub experience_id: String,

    pub factor: ReputationFactor,

    pub impact: f64,

    pub reason: String,
}

impl Reputation {
    pub fn new(id: String) -> Self {
        Self {
            id,
            score: 0.5,
            factors: Vec::new(),
            observations: 0,
            successes: 0,
            failures: 0,
            updated_at: Utc::now(),
            history: Vec::new(),
        }
    }

    pub fn apply(
        &mut self,
        experience_id: String,
        factor: ReputationFactor,
        impact: f64,
        reason: String,
    ) {
        self.score += impact;
        self.score = self.score.clamp(0.0, 1.0);

        self.observations += 1;

        if impact >= 0.0 {
            self.successes += 1;
        } else {
            self.failures += 1;
        }

        self.updated_at = Utc::now();

        self.history.push(ReputationEvent {
            timestamp: Utc::now(),
            experience_id,
            factor,
            impact,
            reason,
        });
    }

    pub fn confidence(&self) -> f64 {
        let amount = self.observations as f64 / 100.0;

        amount.min(1.0)
    }
}
