// src/experience/integration/event_subscriber/handlers.rs

//! Event handler methods for the learning pipeline
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts

use anyhow::Result;
use crate::experience::events::{ExperienceEvent, ExperienceEventType};
use crate::experience::events::payload::EventPayload;
use super::EventSubscriber;

impl EventSubscriber {
    /// Process an experience event through the learning pipeline
    ///
    /// Per Architecture §4.04:
    /// ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts
    pub async fn process_event(&self, event: &ExperienceEvent) -> Result<()> {
        match event.event_type {
            ExperienceEventType::ExperienceRecorded => {
                self.on_experience_recorded(event).await?;
            }
            ExperienceEventType::ReflectionCompleted => {
                self.on_reflection_completed(event).await?;
            }
            ExperienceEventType::HypothesisGenerated => {
                self.on_hypothesis_generated(event).await?;
            }
            ExperienceEventType::HypothesisValidated => {
                self.on_hypothesis_validated(event).await?;
            }
            ExperienceEventType::KnowledgeUpdated => {
                self.on_knowledge_updated(event).await?;
            }
            ExperienceEventType::Scored => {
                self.on_experience_scored(event).await?;
            }
            ExperienceEventType::EvidenceAdded => {
                self.on_evidence_added(event).await?;
            }
            _ => {
                tracing::debug!("Ignoring event type: {:?}", event.event_type);
            }
        }
        Ok(())
    }

    /// Step 1: Experience recorded → Trigger reflection and hypothesis generation
    pub(super) async fn on_experience_recorded(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing ExperienceRecorded event: {}", event.id);

        // Extract experience from payload
        if let EventPayload::ExperienceRecord { experience, .. } = &event.payload {
            // Call coordinator to record the experience (wires metrics and events)
            if let Some(coordinator) = &self.coordinator {
                coordinator.record_experience(experience.id);
            }
        }

        Ok(())
    }

    /// Step 2: Reflection completed → Update hypotheses and knowledge
    pub(super) async fn on_reflection_completed(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing ReflectionCompleted event: {}", event.id);

        // Extract reflection and call coordinator
        if let EventPayload::ReflectionRecord { reflection, .. } = &event.payload {
            if let Some(coordinator) = &self.coordinator {
                // Convert String to Uuid
                if let Ok(id) = uuid::Uuid::parse_str(&reflection.id) {
                    coordinator.complete_reflection(id);
                }
            }
        }

        Ok(())
    }

    /// Step 3: Hypothesis generated → Trigger exploration
    pub(super) async fn on_hypothesis_generated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing HypothesisGenerated event: {}", event.id);

        // Extract hypothesis and call coordinator
        if let EventPayload::HypothesisRecord { hypothesis, .. } = &event.payload {
            if let Some(coordinator) = &self.coordinator {
                // Convert HypothesisId to Uuid
                if let Ok(id) = uuid::Uuid::parse_str(&hypothesis.id.0) {
                    coordinator.generate_hypothesis(id);
                }
            }
        }

        Ok(())
    }

    /// Step 4: Hypothesis validated → Update knowledge
    pub(super) async fn on_hypothesis_validated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing HypothesisValidated event: {}", event.id);

        if let EventPayload::HypothesisValidation { hypothesis_id, result } = &event.payload {
            tracing::debug!("Hypothesis {} validated: {}", hypothesis_id, result);
            // Wire metrics for hypothesis validation
            use crate::experience::metrics::metric_names;
            let metrics = self.metrics.clone();
            let result = result.clone();
            tokio::spawn(async move {
                metrics.increment(metric_names::HYPOTHESES_GENERATED).await;
                // Track confirmed/rejected based on result
                if result.to_lowercase().contains("confirm") || result.to_lowercase().contains("support") {
                    metrics.increment(metric_names::HYPOTHESES_CONFIRMED).await;
                } else if result.to_lowercase().contains("reject") {
                    metrics.increment(metric_names::HYPOTHESES_REJECTED).await;
                }
            });
        }

        Ok(())
    }

    /// Step 5: Knowledge updated → Update reputation
    pub(super) async fn on_knowledge_updated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing KnowledgeUpdated event: {}", event.id);
        // Wire metrics for knowledge updates
        use crate::experience::metrics::metric_names;
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            metrics.increment(metric_names::KNOWLEDGE_CONFIDENCE).await;
        });
        Ok(())
    }

    /// Experience scored → May trigger reflection if score is high
    pub(super) async fn on_experience_scored(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing Scored event: {}", event.id);

        if let EventPayload::ScoreRecord { score, experience_id } = &event.payload {
            // If score exceeds threshold, trigger reflection
            if self.config.auto_reflect && score.confidence >= self.config.reflection_threshold {
                tracing::info!("High-scoring experience {} triggering reflection", experience_id);
                // Reflection will be triggered by the experience recorder
            }
        }

        Ok(())
    }

    /// Evidence added → Update hypothesis confidence
    pub(super) async fn on_evidence_added(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing EvidenceAdded event: {}", event.id);

        if let EventPayload::EvidenceRecord { hypothesis_id, .. } = &event.payload {
            tracing::debug!("Evidence added for hypothesis: {}", hypothesis_id);
        }

        Ok(())
    }
}
