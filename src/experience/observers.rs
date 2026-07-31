// /src/experience/observers.rs
//! Observer implementations for the learning subsystems
//!
//! Per Architecture §22 - Background Workers:
//! Each learning subsystem has a dedicated worker that processes events
//! relevant to its domain.
//!
//! Event Flow:
//! ExperienceRecorded → Reflection → Hypothesis → Knowledge → Reputation
//!                 ↓           ↓           ↓           ↓
//!            Exploration  Evolution   Memory      Sources

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use anyhow::Result;
use chrono::Utc;

use crate::experience::events::ExperienceEvent;
use crate::experience::events::ExperienceEventType;
use crate::experience::events::payload::EventPayload;
use crate::experience::observer::ExperienceObserver;
use crate::experience::reputation::reputation::Reputation;
use crate::experience::reputation::factors::ReputationFactor;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::metrics::MetricsCollector;

/// ============================================================================
/// REPUTATION OBSERVER
/// ============================================================================
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
                let mut reputations = self.reputations.write()
                    .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
                let reputation = reputations.entry(entity_id.clone())
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
                let mut reputations = self.reputations.write()
                    .map_err(|_| anyhow::anyhow!("Lock poisoned"))?;
                let reputation = reputations.entry(entity_id.clone())
                    .or_insert_with(|| Reputation::new(entity_id.clone()));
                reputation.apply(
                    entity_id.clone(),
                    ReputationFactor::Accuracy,
                    *change as f64,
                    "Reputation update from event".to_string(),
                );
                tracing::debug!("ReputationObserver processed reputation change for {}", entity_id);
            }
            EventPayload::ReputationUpdated { previous, current } => {
                tracing::debug!("ReputationObserver: {} -> {}", previous, current);
            }
            _ => {}
        }
        Ok(())
    }
}

/// ============================================================================
/// HYPOTHESIS OBSERVER
/// ============================================================================
/// Processes events to generate and validate hypotheses
pub struct HypothesisObserver {
    engine: Arc<Mutex<HypothesisEngine>>,
}

impl HypothesisObserver {
    pub fn new(engine: Arc<Mutex<HypothesisEngine>>) -> Self {
        Self { engine }
    }
}

impl ExperienceObserver for HypothesisObserver {
    fn name(&self) -> &'static str {
        "HypothesisObserver"
    }

    fn accepts(&self, event: &ExperienceEvent) -> bool {
        matches!(
            event.event_type,
            ExperienceEventType::ExperienceRecorded
                | ExperienceEventType::HypothesisValidated
                | ExperienceEventType::EvidenceAdded
        )
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        match &event.payload {
            EventPayload::ExperienceRecord { experience, .. } => {
                let mut engine = self.engine.lock().unwrap();
                engine.process_experience(experience)?;
                tracing::debug!("HypothesisObserver processed experience: {}", experience.id);
            }
            EventPayload::EvidenceRecord { hypothesis_id, direction, strength, .. } => {
                tracing::debug!("HypothesisObserver: evidence for {} - {} (strength: {})", 
                    hypothesis_id, direction, strength);
            }
            _ => {}
        }
        Ok(())
    }
}

/// ============================================================================
/// METRICS OBSERVER
/// ============================================================================
/// Collects metrics from all events for monitoring
pub struct MetricsObserver {
    metrics: Arc<MetricsCollector>,
}

impl MetricsObserver {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }

    /// Extract metric data from an event and record it
    fn record_event_metrics(&self, event: &ExperienceEvent) {
        let collector = &self.metrics;

        // Increment event counter
        collector.increment_sync(format!("events.{}", event.event_type.name()));

        // Record payload-specific metrics
        match &event.payload {
            EventPayload::ExperienceRecord { experience, .. } => {
                collector.increment_sync("experiences.recorded");

                // Track outcomes
                match experience.outcome.kind {
                    crate::experience::types::OutcomeKind::Success => {
                        collector.increment_sync("experiences.success");
                    }
                    crate::experience::types::OutcomeKind::Failure => {
                        collector.increment_sync("experiences.failure");
                    }
                    crate::experience::types::OutcomeKind::Timeout => {
                        collector.increment_sync("experiences.timeout");
                    }
                    crate::experience::types::OutcomeKind::Interrupted => {
                        collector.increment_sync("experiences.interrupted");
                    }
                    crate::experience::types::OutcomeKind::Partial => {
                        collector.increment_sync("experiences.partial");
                    }
                }

                // Record experience type
                collector.increment_sync(format!(
                    "experiences.type.{}",
                    format!("{:?}", experience.experience_type).to_lowercase()
                ));
            }

            EventPayload::ReflectionRecord { .. } => {
                collector.increment_sync("reflections.created");
            }

            EventPayload::HypothesisRecord { hypothesis } => {
                collector.increment_sync("hypotheses.generated");
                collector.record_sync("hypothesis_created_at", Utc::now().timestamp() as f64);
                // Track initial hypothesis confidence
                collector.record_sync("hypothesis.initial_confidence", hypothesis.confidence.value as f64);
            }

            EventPayload::HypothesisValidation { hypothesis_id, result, .. } => {
                collector.increment_sync("hypotheses.validated");
                // Track validation result
                match result.as_str() {
                    "confirmed" | "supported" => {
                        collector.increment_sync("hypotheses.confirmed");
                    }
                    "rejected" | "contradicted" => {
                        collector.increment_sync("hypotheses.rejected");
                    }
                    _ => {}
                }
                let _ = hypothesis_id;
            }

            EventPayload::EvidenceRecord { hypothesis_id, direction, strength, .. } => {
                collector.increment_sync("evidence.recorded");
                collector.record_sync("evidence.strength", *strength as f64);
                match direction.as_str() {
                    "support" => collector.increment_sync("evidence.supporting"),
                    "contradict" => collector.increment_sync("evidence.contradicting"),
                    _ => collector.increment_sync("evidence.neutral"),
                }
                let _ = hypothesis_id;
            }

            EventPayload::KnowledgeRecord { .. } => {
                collector.increment_sync("knowledge.created");
            }

            EventPayload::Reputation { .. } => {
                collector.increment_sync("reputation.updates");
            }

            EventPayload::ExplorationRecord { .. } => {
                collector.increment_sync("explorations.completed");
            }

            EventPayload::ExplorationCompleted { exploration_id, .. } => {
                collector.increment_sync("explorations.completed");
                let _ = exploration_id;
            }

            EventPayload::ScoreCalculated { score } => {
                collector.record_sync("experience.score.calculated", *score as f64);
            }

            EventPayload::ObserverStarted { observer } => {
                collector.increment_sync("observers.started");
                collector.record_sync("observer_started", Utc::now().timestamp() as f64);
                let _ = observer;
            }

            EventPayload::ObserverFailed { observer, .. } => {
                collector.increment_sync("observers.failed");
                let _ = observer;
            }

            EventPayload::ProcessingFailed { stage, .. } => {
                collector.increment_sync("processing.failed");
                collector.increment_sync(format!("processing.failed.{}", stage));
            }

            EventPayload::Error { .. } => {
                collector.increment_sync("events.errors");
            }

            _ => {}
        }
    }
}

impl ExperienceObserver for MetricsObserver {
    fn name(&self) -> &'static str {
        "MetricsObserver"
    }

    #[allow(unused)]
    fn accepts(&self, _event: &ExperienceEvent) -> bool {
        true
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        self.record_event_metrics(event);
        Ok(())
    }
}
