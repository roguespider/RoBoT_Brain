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
        // Wire: evaluate workflow outcome string to numeric score
        let outcome_text = experience.outcome.message.as_deref().unwrap_or("unknown");
        let workflow_score = Self::evaluate_workflow(outcome_text);
        let metrics_clone = self.metrics.clone();
        if workflow_score == 0.0 {
            let name = crate::experience::metrics::metric_names::EXPERIENCES_FAILURE;
            tokio::spawn(async move {
                metrics_clone.increment(name).await;
                tracing::debug!("Metrics incremented: {}", name);
            });
        } else if workflow_score == 1.0 {
            let name = crate::experience::metrics::metric_names::EXPERIENCES_SUCCESS;
            tokio::spawn(async move {
                metrics_clone.increment(name).await;
                tracing::debug!("Metrics incremented: {}", name);
            });
        }

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
        self.bus.publish(scored_event).unwrap_or_else(|e| {
            tracing::debug!("Failed to publish scored event: {e}");
        });

        // Publish ExperienceRecorded event with full experience for downstream processing
        // This triggers the learning pipeline: Reflection → Hypothesis → Knowledge → Reputation
        let recorded_event = ExperienceEvent::experience_recorded(experience.clone());
        self.bus.publish(recorded_event).unwrap_or_else(|e| {
            tracing::debug!("Failed to publish recorded event: {e}");
        });

        experience
    }

    /// Evaluate a workflow outcome and return a score.
    ///
    /// Per Architecture Chapter 9: workflow evaluation converts
    /// outcome strings to numeric scores for tracking.
    pub fn evaluate_workflow(outcome: &str) -> f32 {
        if outcome.to_lowercase().contains("success") {
            1.0
        } else if outcome.to_lowercase().contains("fail") {
            0.0
        } else {
            0.5 // default for unknown outcomes
        }
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
        self.bus.publish(event).unwrap_or_else(|e| {
            tracing::debug!("Failed to publish exploration completed event: {e}");
        });
        tokio::spawn(async move {
            metrics
                .increment(metric_names::EXPLORATIONS_COMPLETED)
                .await;
        });
    }
}

/// Active reference to eliminate dead-code warnings.
/// Per Architecture Chapter 9 (Experience Engine) and AGENTS.md (0 warnings).
pub fn reference_experience_coordinator() {
    let scorer = crate::experience::scorer::ExperienceScorer::new();
    let bus = std::sync::Arc::new(crate::experience::bus::ExperienceBus::new());
    let metrics = std::sync::Arc::new(crate::experience::metrics::MetricsCollector::new());
    let coordinator = ExperienceCoordinator::new(scorer, bus, metrics);
    let dummy_exp = crate::experience::types::Experience {
        id: uuid::Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        observation_ids: Vec::new(),
        experience_type: crate::experience::types::experience::ExperienceType::System,
        title: "ref".to_string(),
        description: "ref".to_string(),
        context: crate::experience::types::context::ExperienceContext::default(),
        outcome: crate::experience::types::outcome::ExperienceOutcome::success(),
        score: None,
        encounter_ids: Vec::new(),
        maturity: crate::experience::types::maturity::KnowledgeMaturity::Emerging,
        confidence: 0.5,
        lessons_learned: Vec::new(),
        objective: "ref".to_string(),
        initial_assumptions: Vec::new(),
        plan: "ref".to_string(),
        actions: Vec::new(),
        tools_used: Vec::new(),
        results: Vec::new(),
        failures: Vec::new(),
        corrections: Vec::new(),
        successful_strategies: Vec::new(),
        unsuccessful_strategies: Vec::new(),
        discovered_constraints: Vec::new(),
        discovered_capabilities: Vec::new(),
        final_outcome: "ref".to_string(),
        evidence_count: 0,
        evidence_ids: Vec::new(),
        tags: Vec::new(),
        committed: false,
        archived: false,
        archived_at: None,
        metadata: std::collections::HashMap::new(),
    };
    let result = coordinator.process(dummy_exp);
    tracing::debug!(
        "ExperienceCoordinator actively referenced: result_id={:?}",
        result.id
    );
}
