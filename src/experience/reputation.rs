// /src/experience/reputation.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tracks trust accumulated through experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    pub id: String,

    /// Current reputation score.
    pub score: f64,

    /// Number of observations contributing to score.
    pub observations: u64,

    /// Successful outcomes.
    pub successes: u64,

    /// Failed outcomes.
    pub failures: u64,

    /// Last time reputation changed.
    pub updated_at: DateTime<Utc>,

    /// Historical changes.
    pub history: Vec<ReputationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: DateTime<Utc>,

    pub experience_id: String,

    pub impact: f64,

    pub reason: String,
}

impl Reputation {
    pub fn new(id: String) -> Self {
        Self {
            id,
            score: 0.5,
            observations: 0,
            successes: 0,
            failures: 0,
            updated_at: Utc::now(),
            history: Vec::new(),
        }
    }

    pub fn record_success(&mut self, experience_id: String, reason: String) {
        self.apply_change(experience_id, 0.02, reason);

        self.successes += 1;
    }

    pub fn record_failure(&mut self, experience_id: String, reason: String) {
        self.apply_change(experience_id, -0.03, reason);

        self.failures += 1;
    }

    fn apply_change(&mut self, experience_id: String, impact: f64, reason: String) {
        self.score += impact;

        self.score = self.score.clamp(0.0, 1.0);

        self.observations += 1;
        self.updated_at = Utc::now();

        self.history.push(ReputationEvent {
            timestamp: Utc::now(),
            experience_id,
            impact,
            reason,
        });
    }

    pub fn confidence(&self) -> f64 {
        match self.observations {
            0 => 0.0,
            n => {
                let confidence = n as f64 / 100.0;
                confidence.min(1.0)
            }
        }
    }
}
