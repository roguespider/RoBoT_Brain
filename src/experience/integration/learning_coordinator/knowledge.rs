// src/experience/integration/learning_coordinator/knowledge.rs
//! Knowledge management methods

use std::sync::Arc;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::metrics::MetricsCollector;
use crate::experience::types::Experience;
use crate::knowledge::{KnowledgeItem, KnowledgeStore};
use uuid::Uuid;

/// Knowledge methods for LearningCoordinator
pub struct KnowledgeMethods<'a> {
    pub knowledge_store: &'a Arc<KnowledgeStore>,
    pub metrics: &'a Arc<MetricsCollector>,
    pub bus: &'a Arc<ExperienceBus>,
}

impl<'a> KnowledgeMethods<'a> {
    /// Promote high-value experience to knowledge
    ///
    /// Per Architecture §2.3:
    /// "Knowledge is information that has survived evaluation"
    pub async fn promote_to_knowledge(&self, experience: &Experience) -> Result<(), anyhow::Error> {
        let knowledge = KnowledgeItem::from_reflection(
            &experience.description,
            experience.confidence,
            experience.id,
        );

        let _ = self.knowledge_store.add(knowledge).await;

        // Publish KnowledgeUpdated event
        let event = ExperienceEvent::knowledge_updated(Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish knowledge event: {}", e);
        }

        Ok(())
    }

    /// Consolidate low-confidence knowledge
    pub async fn consolidate_knowledge(&self) -> Result<usize, anyhow::Error> {
        tracing::debug!("Running knowledge consolidation");

        // Find and consolidate low-confidence knowledge
        let mut consolidated = 0;
        let knowledge_items = self.knowledge_store.get_all().await;

        for knowledge in knowledge_items {
            // If knowledge confidence is below threshold, try to consolidate
            let overall_confidence = knowledge.confidence.overall();
            if overall_confidence < 0.3 {
                // Archive or remove low-confidence knowledge
                // For now, just log it
                tracing::debug!(
                    "Low-confidence knowledge {} found with confidence {:.2}",
                    knowledge.id,
                    overall_confidence
                );
                consolidated += 1;
            }
        }

        Ok(consolidated)
    }
}
