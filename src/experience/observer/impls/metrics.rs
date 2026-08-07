// src/experience/observer/impls/metrics.rs
//! Metrics Observer implementation

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

use crate::experience::events::payload::EventPayload;
use crate::experience::events::ExperienceEvent;
use crate::experience::metrics::MetricsCollector;
use crate::experience::observer::ExperienceObserver;

/// Metrics Observer implementation
///
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
                collector.record_sync(
                    "hypothesis.initial_confidence",
                    hypothesis.confidence.value as f64,
                );
            }

            EventPayload::HypothesisValidation {
                hypothesis_id,
                result,
                ..
            } => {
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
                let _hypothesis_id = hypothesis_id;
            }

            EventPayload::EvidenceRecord {
                hypothesis_id,
                direction,
                strength,
                ..
            } => {
                collector.increment_sync("evidence.recorded");
                collector.record_sync("evidence.strength", *strength as f64);
                match direction.as_str() {
                    "support" => collector.increment_sync("evidence.supporting"),
                    "contradict" => collector.increment_sync("evidence.contradicting"),
                    _ => collector.increment_sync("evidence.neutral"),
                }
                let _hypothesis_id = hypothesis_id;
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
                let _exploration_id = exploration_id;
            }

            EventPayload::ScoreCalculated { score } => {
                collector.record_sync("experience.score.calculated", *score as f64);
            }

            EventPayload::ObserverStarted { observer } => {
                collector.increment_sync("observers.started");
                collector.record_sync("observer_started", Utc::now().timestamp() as f64);
                let _observer = observer;
            }

            EventPayload::ObserverFailed { observer, .. } => {
                collector.increment_sync("observers.failed");
                let _observer = observer;
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

    fn accepts(&self, event: &ExperienceEvent) -> bool {
        // MetricsObserver accepts all event types to collect comprehensive metrics.
        // System and custom events are still tracked but may be filtered in the observe method.
        tracing::trace!(
            "MetricsObserver checking event type: {}",
            event.event_type.name()
        );
        true
    }

    fn observe(&self, event: &ExperienceEvent) -> Result<()> {
        self.record_event_metrics(event);
        Ok(())
    }
}
