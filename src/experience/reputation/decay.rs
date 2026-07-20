// /src/experience/reputation/decay.rs

use chrono::{DateTime, Utc};

pub struct ReputationDecay;

impl ReputationDecay {
    pub fn apply(score: f64, updated: DateTime<Utc>) -> f64 {
        let days = (Utc::now() - updated).num_days();

        let decay = days as f64 * 0.001;

        let result = score - decay;

        result.clamp(0.0, 1.0)
    }
}
