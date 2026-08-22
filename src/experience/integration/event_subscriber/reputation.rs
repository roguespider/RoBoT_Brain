// src/experience/integration/event_subscriber/reputation.rs

//! Reputation management for the event subscriber

use anyhow::Result;
use super::EventSubscriber;

impl EventSubscriber {
    /// Record reputation for a source
    pub async fn record_reputation(
        &self,
        source_id: &str,
        impact: f64,
        reason: &str,
    ) -> Result<()> {
        let mut store = self.reputation_store.write().await;
        let reputation = store.entry(source_id.to_string())
            .or_insert_with(|| crate::experience::reputation::score::Reputation::new(source_id.to_string()));
        
        reputation.apply(
            String::new(), // No specific experience
            crate::experience::reputation::factors::ReputationFactor::Accuracy,
            impact,
            reason.to_string(),
        );

        Ok(())
    }

    /// Get reputation score for a source
    pub async fn get_reputation(&self, source_id: &str) -> Option<f64> {
        let store = self.reputation_store.read().await;
        store.get(source_id).map(|r| r.score)
    }
}
