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

use crate::experience::events::ExperienceEvent;
use crate::experience::events::ExperienceEventType;
use crate::experience::events::payload::EventPayload;
use crate::experience::observer::ExperienceObserver;
use crate::experience::reputation::reputation::Reputation;
use crate::experience::reputation::factors::ReputationFactor;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::evolution::EvolutionEngine;
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
}

impl ExperienceObserver for MetricsObserver {
    fn name(&self) -> &'static str {
        "MetricsObserver"
    }

    fn accepts(&self, _event: &ExperienceEvent) -> bool {
        true
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        // Note: MetricsCollector uses async methods, so in synchronous context
        // we just log the event. In practice, the metrics would be recorded
        // via async calls from the worker.
        let event_name = event.event_type.name();
        tracing::trace!("MetricsObserver recorded event: {}", event_name);
        Ok(())
    }
}
