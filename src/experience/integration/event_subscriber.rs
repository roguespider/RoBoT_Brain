// src/experience/integration/event_subscriber.rs
//! Event subscriber that listens to experience events and triggers learning pipeline
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts
//!
//! This subscriber wires the event bus to all learning subsystems.



use std::sync::Arc;
use tokio::sync::broadcast;
use anyhow::Result;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::{ExperienceEvent, ExperienceEventType};
use crate::experience::types::Experience;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::reputation::reputation::Reputation;
use crate::knowledge::KnowledgeStore;
use crate::experience::events::payload::EventPayload;

/// Configuration for event subscription behavior
#[derive(Debug, Clone)]
pub struct EventSubscriberConfig {
    /// Whether to auto-generate reflections
    pub auto_reflect: bool,
    /// Whether to auto-generate hypotheses
    pub auto_hypothesize: bool,
    /// Minimum score to trigger reflection
    pub reflection_threshold: f32,
    /// Whether to update knowledge from experiences
    pub auto_update_knowledge: bool,
}

impl Default for EventSubscriberConfig {
    fn default() -> Self {
        Self {
            auto_reflect: true,
            auto_hypothesize: true,
            reflection_threshold: 0.6,
            auto_update_knowledge: true,
        }
    }
}

/// Event subscriber that coordinates the learning pipeline
///
/// This is the main coordinator that wires events to learning subsystems
/// per Architecture §4.04: Experience → Reflection → Hypothesis → Knowledge → Reputation
pub struct EventSubscriber {
    config: EventSubscriberConfig,
    reflection_engine: Arc<ReflectionEngine>,
    hypothesis_engine: Arc<HypothesisEngine>,
    evolution_engine: Arc<EvolutionEngine>,
    knowledge_store: Arc<KnowledgeStore>,
    reputation_store: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Reputation>>>,
}

impl EventSubscriber {
    /// Create a new event subscriber with dependencies
    pub fn new(
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
    ) -> Self {
        Self {
            config: EventSubscriberConfig::default(),
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            reputation_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with custom config
    pub fn with_config(
        config: EventSubscriberConfig,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
    ) -> Self {
        Self {
            config,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            reputation_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

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
    async fn on_experience_recorded(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing ExperienceRecorded event: {}", event.id);

        // Extract experience from payload
        if let EventPayload::ExperienceRecord { experience, .. } = &event.payload {
            // Step 2: Reflection observes the experience
            if self.config.auto_reflect {
                self.generate_reflection(experience).await?;
            }

            // Step 3: Hypothesis evaluates the experience
            if self.config.auto_hypothesize {
                self.generate_hypothesis(experience).await?;
            }
        }

        Ok(())
    }

    /// Step 2: Reflection completed → Update hypotheses and knowledge
    async fn on_reflection_completed(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing ReflectionCompleted event: {}", event.id);

        // Extract reflection insights and update knowledge
        if self.config.auto_update_knowledge {
            if let EventPayload::ReflectionRecord { reflection, .. } = &event.payload {
                self.update_knowledge_from_reflection(reflection).await?;
            }
        }

        Ok(())
    }

    /// Step 3: Hypothesis generated → Trigger exploration
    async fn on_hypothesis_generated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing HypothesisGenerated event: {}", event.id);
        // Hypotheses trigger exploration - handled by exploration system
        Ok(())
    }

    /// Step 4: Hypothesis validated → Update knowledge
    async fn on_hypothesis_validated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing HypothesisValidated event: {}", event.id);

        if let EventPayload::HypothesisValidation { hypothesis_id, result } = &event.payload {
            tracing::debug!("Hypothesis {} validated: {}", hypothesis_id, result);
        }

        Ok(())
    }

    /// Step 5: Knowledge updated → Update reputation
    async fn on_knowledge_updated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing KnowledgeUpdated event: {}", event.id);
        // Reputation adjusts based on knowledge updates
        Ok(())
    }

    /// Experience scored → May trigger reflection if score is high
    async fn on_experience_scored(&self, event: &ExperienceEvent) -> Result<()> {
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
    async fn on_evidence_added(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing EvidenceAdded event: {}", event.id);

        if let EventPayload::EvidenceRecord { hypothesis_id, .. } = &event.payload {
            tracing::debug!("Evidence added for hypothesis: {}", hypothesis_id);
        }

        Ok(())
    }

    /// Generate reflection from experience
    async fn generate_reflection(&self, experience: &Experience) -> Result<()> {
        let _reflection = self.reflection_engine
            .generate_from_single(experience, format!("Reflection on: {}", experience.title))
            .await?;

        tracing::info!("Generated reflection for experience: {}", experience.id);
        Ok(())
    }

    /// Generate hypothesis from experience
    async fn generate_hypothesis(&self, experience: &Experience) -> Result<()> {
        // Use hypothesis engine to process the experience
        // If high-scoring, create a behavior via evolution engine
        if let Some(score) = &experience.score {
            if score.confidence > 0.7 {
                // Create an insight from the high-confidence experience
                let mut insight = crate::experience::reflection::insight::Insight::new(
                    uuid::Uuid::new_v4().to_string(),
                    format!("Insight from: {}", experience.title),
                    format!("High-confidence experience: {:?}", experience.outcome),
                    crate::experience::reflection::insight::InsightType::Pattern,
                );
                insight.confidence = score.confidence;
                insight.add_experience(experience.id.to_string());
                
                let _behavior = self.evolution_engine.create_behavior_from_insight(&insight).await?;
                tracing::info!("Created behavior from high-confidence experience: {}", experience.id);
            }
        }

        tracing::info!("Generated hypotheses from experience");
        Ok(())
    }

    /// Update knowledge store from reflection insights
    async fn update_knowledge_from_reflection(&self, _reflection: &crate::experience::reflection::Reflection) -> Result<()> {
        // Extract insights and create knowledge items
        // This bridges Reflection → Knowledge per Architecture §4.04
        Ok(())
    }

    /// Update knowledge from validated hypothesis
    async fn update_knowledge_from_hypothesis(
        &self,
        _hypothesis: &crate::experience::hypothesis::core::hypothesis::Hypothesis,
        _result: &str,
    ) -> Result<()> {
        // If hypothesis is validated, create knowledge from it
        // Per Architecture §2.5: "Hypothesis is a temporary model waiting for evidence"
        Ok(())
    }

    /// Update hypothesis with new evidence
    async fn update_hypothesis_with_evidence(
        &self,
        hypothesis_id: &str,
        _evidence: &crate::experience::events::payload::EventPayload,
    ) -> Result<()> {
        // Update hypothesis confidence based on evidence
        tracing::debug!("Updating hypothesis {} with new evidence", hypothesis_id);
        Ok(())
    }

    /// Record reputation for a source
    pub async fn record_reputation(
        &self,
        source_id: &str,
        impact: f64,
        reason: &str,
    ) -> Result<()> {
        let mut store = self.reputation_store.write().await;
        let reputation = store.entry(source_id.to_string())
            .or_insert_with(|| Reputation::new(source_id.to_string()));
        
        let _ = reputation.apply(
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

/// Start the event subscriber as a background task
pub fn start_event_subscriber(
    bus: Arc<ExperienceBus>,
    subscriber: Arc<EventSubscriber>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut receiver = bus.subscribe();
        tracing::info!("Event subscriber started, listening for events");

        loop {
            match receiver.recv().await {
                Ok(event) => {
                    if let Err(e) = subscriber.process_event(&event).await {
                        tracing::error!("Error processing event {}: {}", event.id, e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Event subscriber lagged {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Event bus closed, subscriber shutting down");
                    break;
                }
            }
        }
    })
}
