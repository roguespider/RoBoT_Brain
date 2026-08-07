// src/experience/observer/impls/hypothesis.rs
//! Hypothesis Observer implementation

use anyhow::Result;
use std::sync::{Arc, Mutex};

use crate::experience::events::payload::EventPayload;
use crate::experience::events::ExperienceEvent;
use crate::experience::events::ExperienceEventType;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::observer::ExperienceObserver;

/// Hypothesis Observer implementation
///
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
                match self.engine.lock() {
                    Ok(mut engine) => {
                        engine.process_experience(experience)?;
                        tracing::debug!("HypothesisObserver processed experience: {}", experience.id);
                    }
                    Err(poisoned) => {
                        tracing::error!("HypothesisEngine mutex poisoned during observe");
                        let mut engine = poisoned.into_inner();
                        if let Err(e) = engine.process_experience(experience) {
                            tracing::error!("Failed to process experience on recovered mutex: {}", e);
                        }
                    }
                }
            }
            EventPayload::EvidenceRecord {
                hypothesis_id,
                direction,
                strength,
                ..
            } => {
                tracing::debug!(
                    "HypothesisObserver: evidence for {} - {} (strength: {})",
                    hypothesis_id,
                    direction,
                    strength
                );
            }
            _ => {}
        }
        Ok(())
    }
}
