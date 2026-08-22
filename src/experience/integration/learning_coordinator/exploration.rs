// src/experience/integration/learning_coordinator/exploration.rs
//! Exploration management methods

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::exploration::Exploration;
use chrono::{Duration, Utc};

/// Exploration manager for the learning coordinator
pub struct ExplorationManager {
    explorations: Arc<RwLock<std::collections::HashMap<String, Exploration>>>,
    bus: Arc<ExperienceBus>,
}

impl ExplorationManager {
    pub fn new(
        explorations: Arc<RwLock<std::collections::HashMap<String, Exploration>>>,
        bus: Arc<ExperienceBus>,
    ) -> Self {
        Self { explorations, bus }
    }

    /// Start exploration for a hypothesis
    pub async fn start_exploration(
        &self,
        hypothesis_id: String,
        title: String,
        purpose: String,
    ) -> Result<String, anyhow::Error> {
        let exploration_id = Uuid::new_v4().to_string();

        // Link exploration to the hypothesis it's investigating
        let mut exploration = Exploration::new(
            exploration_id.clone(),
            title,
            purpose,
            crate::experience::types::ExperienceContext {
                related_hypothesis: Some(hypothesis_id.clone()),
                ..Default::default()
            },
        );
        // Activations start immediately: an exploration created by the
        // coordinator is by definition being pursued (Architecture §4.06).
        exploration.start();

        let mut store = self.explorations.write().await;
        store.insert(exploration_id.clone(), exploration);

        // Publish ExplorationStarted event
        let event = ExperienceEvent::exploration_started(Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish exploration event: {}", e);
        }

        Ok(exploration_id)
    }

    /// Complete an exploration
    pub async fn complete_exploration(&self, exploration_id: &str) -> Result<(), anyhow::Error> {
        let mut store = self.explorations.write().await;

        if let Some(exp) = store.get_mut(exploration_id) {
            exp.complete();
        }

        // Publish ExplorationCompleted event
        let event = ExperienceEvent::exploration_completed(Uuid::new_v4(), Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish exploration completed event: {}", e);
        }

        Ok(())
    }

    /// Archive stale explorations
    pub async fn archive_stale_explorations(&self) -> Result<usize, anyhow::Error> {
        let cutoff = Utc::now() - Duration::days(7);
        let mut store = self.explorations.write().await;
        let mut archived = 0;

        store.retain(|id, exp| {
            if let Some(completed) = exp.completed_at {
                tracing::trace!("Archiving exploration {}", id);
                if completed < cutoff {
                    archived += 1;
                    return false;
                }
            }
            true
        });

        Ok(archived)
    }

    /// Get count of active explorations
    pub async fn active_count(&self) -> usize {
        let store = self.explorations.read().await;
        store.values().filter(|e| e.is_active()).count()
    }
}
