// src/experience/observer/impls/reputation.rs
//! Reputation Observer implementation

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::experience::events::payload::EventPayload;
use crate::experience::events::ExperienceEvent;
use crate::experience::events::ExperienceEventType;
use crate::experience::observer::ExperienceObserver;
use crate::experience::reputation::factors::ReputationFactor;
use crate::experience::reputation::score::Reputation;

/// Reputation Observer implementation
///
/// Processes events that affect entity reputation scores
pub struct ReputationObserver {
    reputations: Arc<RwLock<HashMap<String, Reputation>>>,
}

impl ReputationObserver {
    pub fn new() -> Self {
        Self {
            reputations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for ReputationObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperienceObserver for ReputationObserver {
    fn name(&self) -> &'static str {
        "ReputationObserver"
    }

    fn accepts(&self, event: &ExperienceEvent) -> bool {
        matches!(
            event.event_type,
            ExperienceEventType::ExperienceRecorded
                | ExperienceEventType::ReputationUpdated
                | ExperienceEventType::KnowledgeUpdated
        )
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        match &event.payload {
            EventPayload::ExperienceRecord { experience, .. } => {
                let entity_id = format!("experience_{}", experience.id);
                let mut reputations = self
                    .reputations
                    .write()
                    .map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
                let reputation = reputations
                    .entry(entity_id.clone())
                    .or_insert_with(|| Reputation::new(entity_id.clone()));

                let delta = match experience.outcome.kind {
                    crate::experience::types::OutcomeKind::Success => 0.1,
                    crate::experience::types::OutcomeKind::Failure => -0.1,
                    _ => 0.0,
                };

                if delta != 0.0 {
                    reputation.apply(
                        entity_id.clone(),
                        ReputationFactor::Accuracy,
                        delta,
                        format!("Experience outcome: {:?}", experience.outcome.kind),
                    );
                    tracing::debug!("ReputationObserver updated reputation for {}", entity_id);
                }
            }
            EventPayload::Reputation { entity_id, change } => {
                let mut reputations = self
                    .reputations
                    .write()
                    .map_err(|e| anyhow::anyhow!("Lock poisoned: {:?}", e))?;
                let reputation = reputations
                    .entry(entity_id.clone())
                    .or_insert_with(|| Reputation::new(entity_id.clone()));
                reputation.apply(
                    entity_id.clone(),
                    ReputationFactor::Accuracy,
                    *change as f64,
                    "Reputation update from event".to_string(),
                );
                tracing::debug!(
                    "ReputationObserver processed reputation change for {}",
                    entity_id
                );
            }
            EventPayload::ReputationUpdated { previous, current } => {
                tracing::debug!("ReputationObserver: {} -> {}", previous, current);
            }
            _ => {}
        }
        Ok(())
    }
}
