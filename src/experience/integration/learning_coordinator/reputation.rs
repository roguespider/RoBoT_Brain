// src/experience/integration/learning_coordinator/reputation.rs
//! Reputation management methods

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::reputation::factors::ReputationFactor;
use crate::experience::reputation::reputation::Reputation;
use crate::experience::types::Experience;
use crate::experience::types::outcome::OutcomeKind;

/// Reputation manager for the learning coordinator
pub struct ReputationManager {
    reputations: Arc<RwLock<std::collections::HashMap<String, Reputation>>>,
    bus: Arc<ExperienceBus>,
}

impl ReputationManager {
    pub fn new(reputations: Arc<RwLock<std::collections::HashMap<String, Reputation>>>, bus: Arc<ExperienceBus>) -> Self {
        Self { reputations, bus }
    }

    /// Update reputation based on experience outcome
    ///
    /// Per Architecture §12:
    /// "Reputation determines how much each source of knowledge should be trusted"
    pub async fn update_reputation(&self, experience: &Experience) -> Result<(), anyhow::Error> {
        let source = &experience.context.source;
        let source_str = match source {
            Some(s) => s.clone(),
            None => return Ok(()),
        };

        if source_str.is_empty() {
            return Ok(());
        }

        let mut store = self.reputations.write().await;
        let reputation = store
            .entry(source_str.clone())
            .or_insert_with(|| Reputation::new(source_str.clone()));

        // Determine impact based on outcome
        let (impact, reason) = match experience.outcome.kind {
            OutcomeKind::Success => (0.1, "Successful experience".to_string()),
            OutcomeKind::Partial => (0.0, "Partial success".to_string()),
            OutcomeKind::Failure => (-0.15, "Failed experience".to_string()),
            OutcomeKind::Interrupted => (-0.05, "Interrupted".to_string()),
            _ => (0.0, "Unknown outcome".to_string()),
        };

        reputation.apply(
            experience.id.to_string(),
            ReputationFactor::Accuracy,
            impact,
            reason,
        );

        // Publish ReputationUpdated event
        let event = ExperienceEvent::reputation_updated(Uuid::new_v4(), source_str, impact as f32);
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish reputation event: {}", e);
        }

        Ok(())
    }

    /// Decay reputations over time
    pub async fn decay_reputations(&self) -> Result<(), anyhow::Error> {
        let mut store = self.reputations.write().await;

        for reputation in store.values_mut() {
            // Apply small decay
            if reputation.score > 0.5 {
                reputation.score = (reputation.score - 0.01).max(0.5);
            } else if reputation.score < 0.5 {
                reputation.score = (reputation.score + 0.01).min(0.5);
            }
        }

        Ok(())
    }

    /// Get reputation for a source
    pub async fn get_reputation(&self, source: &str) -> Option<f64> {
        let store = self.reputations.read().await;
        store.get(source).map(|r| r.score)
    }

    /// Get count of active reputations
    pub async fn active_count(&self) -> usize {
        let store = self.reputations.read().await;
        store.len()
    }
}
