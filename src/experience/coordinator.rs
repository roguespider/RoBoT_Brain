// /src/experience/coordinator.rs

// Experience system coordinator per Architecture §07

use crate::experience::{
    bus::ExperienceBus, events::ExperienceEvent, metrics::MetricsCollector,
    scorer::ExperienceScorer, types::*,
};
use std::sync::Arc;
use uuid::Uuid;

/// Coordinates the experience system.
///
/// The manager does not contain business logic.
/// Instead it orchestrates the specialized components.
pub struct ExperienceCoordinator {
    scorer: ExperienceScorer,
    bus: Arc<ExperienceBus>,
    metrics: Arc<MetricsCollector>,
}

impl ExperienceCoordinator {
    pub fn new(
        scorer: ExperienceScorer,
        bus: Arc<ExperienceBus>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            scorer,
            bus,
            metrics,
        }
    }

    /// Process a completed experience through the learning pipeline.
    ///
    /// This method:
    /// 1. Scores the experience
    /// 2. Records metrics
    /// 3. Publishes Scored event
    /// 4. Publishes ExperienceRecorded event (with full experience for downstream processing)
    pub fn process(&self, mut experience: Experience) -> Experience {
        // Score it.
        let score = self.scorer.score(&experience);
        experience.score = Some(score.clone());

        // Record metrics
        use crate::experience::metrics::metric_names;
        let metrics_clone = self.metrics.clone();
        let outcome_kind = experience.outcome.kind;
        tokio::spawn(async move {
            metrics_clone
                .increment(metric_names::EXPERIENCES_RECORDED)
                .await;
            match outcome_kind {
                OutcomeKind::Success | OutcomeKind::Partial => {
                    metrics_clone
                        .increment(metric_names::EXPERIENCES_SUCCESS)
                        .await;
                }
                OutcomeKind::Failure => {
                    metrics_clone
                        .increment(metric_names::EXPERIENCES_FAILURE)
                        .await;
                }
                _ => {}
            }
        });

        // Publish Scored event
        let scored_event = ExperienceEvent::scored(experience.id, score.clone());
        let _ = self.bus.publish(scored_event);

        // Publish ExperienceRecorded event with full experience for downstream processing
        // This triggers the learning pipeline: Reflection → Hypothesis → Knowledge → Reputation
        let recorded_event = ExperienceEvent::experience_recorded(experience.clone());
        let _ = self.bus.publish(recorded_event);

        experience
    }

    /// Record that exploration was completed
    pub fn complete_exploration(&self, id: &str) {
        use crate::experience::metrics::metric_names;
        let metrics = self.metrics.clone();
        let exploration_id = Uuid::new_v4();
        let event = ExperienceEvent::exploration_completed(
            exploration_id,
            Uuid::parse_str(id).unwrap_or_default(),
        );
        let _ = self.bus.publish(event);
        tokio::spawn(async move {
            metrics
                .increment(metric_names::EXPLORATIONS_COMPLETED)
                .await;
        });
    }
}
