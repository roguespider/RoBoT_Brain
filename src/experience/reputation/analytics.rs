// /src/experience/reputation/analytics.rs

use super::reputation::Reputation;

pub struct ReputationAnalytics;

impl ReputationAnalytics {
    pub fn success_rate(reputation: &Reputation) -> f64 {
        if reputation.observations == 0 {
            return 0.0;
        }

        reputation.successes as f64 / reputation.observations as f64
    }

    pub fn trend(reputation: &Reputation) -> f64 {
        if reputation.history.len() < 2 {
            return 0.0;
        }

        let first = reputation.history.first()
            .map(|h| h.impact)
            .unwrap_or(0.0);

        let last = reputation.history.last()
            .map(|h| h.impact)
            .unwrap_or(0.0);

        last - first
    }
}
