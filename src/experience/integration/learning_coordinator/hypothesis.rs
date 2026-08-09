// src/experience/integration/learning_coordinator/hypothesis.rs
//! Hypothesis generation and management methods

use std::sync::Arc;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::hypothesis::core::hypothesis::{Hypothesis, HypothesisCategory, HypothesisConfidence};
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::types::Experience;
use crate::experience::types::outcome::OutcomeKind;
use uuid::Uuid;

use super::config::LearningCoordinatorConfig;

/// Hypothesis methods for LearningCoordinator
pub struct HypothesisMethods<'a> {
    pub config: &'a LearningCoordinatorConfig,
    pub hypothesis_engine: &'a Arc<HypothesisEngine>,
    pub reflection_engine: &'a Arc<ReflectionEngine>,
    pub metrics: &'a Arc<MetricsCollector>,
    pub bus: &'a Arc<ExperienceBus>,
}

impl<'a> HypothesisMethods<'a> {
    /// Generate hypotheses from experience
    ///
    /// Per Architecture §11:
    /// "Hypotheses enable discovery"
    pub async fn generate_hypotheses(&self, experience: &Experience) -> Result<Vec<String>, anyhow::Error> {
        let mut hypothesis_ids = Vec::new();

        // Generate a hypothesis from the experience
        let hypothesis = self.create_hypothesis_from_experience(experience).await;

        if let Some(h) = hypothesis {
            // Only retain hypotheses that clear the validation threshold
            // configured for this coordinator (Architecture §11).
            if h.confidence.value >= self.config.hypothesis_validation_threshold {
                hypothesis_ids.push(h.id.0.clone());
                tracing::info!(
                    "Generated hypothesis {} from experience {}",
                    h.id.0,
                    experience.id
                );
            }
        }

        // Publish HypothesisGenerated event
        let event = ExperienceEvent::hypothesis_generated(experience.id, Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish hypothesis event: {}", e);
        }

        // Record hypothesis generation metric (Architecture §22 observability).
        if !hypothesis_ids.is_empty() {
            self.metrics
                .increment(crate::experience::metrics::metric_names::HYPOTHESES_GENERATED)
                .await;
        }

        Ok(hypothesis_ids)
    }

    /// Create a hypothesis from an experience
    pub async fn create_hypothesis_from_experience(
        &self,
        experience: &Experience,
    ) -> Option<Hypothesis> {
        use crate::experience::types::ExperienceType;

        // Create hypothesis based on experience type and outcome
        let title = format!(
            "{}: {}",
            match experience.outcome.kind {
                OutcomeKind::Success => "What worked",
                OutcomeKind::Failure => "What failed",
                _ => "Observation",
            },
            experience.title
        );

        let statement = format!(
            "{} - This {} resulted in {}",
            experience.description,
            match experience.experience_type {
                ExperienceType::ToolExecution => "tool execution",
                ExperienceType::Planning => "planning activity",
                ExperienceType::Workflow => "workflow step",
                _ => "action",
            },
            match experience.outcome.kind {
                OutcomeKind::Success => "success",
                OutcomeKind::Failure => "failure",
                _ => "an uncertain outcome",
            }
        );

        // Calculate initial confidence based on experience confidence and evidence
        let mut confidence = experience.confidence;
        if experience.evidence_count > 0 {
            confidence = (confidence * 0.7) + ((experience.evidence_count as f32 * 0.05).min(0.3));
        }

        let mut hypothesis = Hypothesis::new(title, statement);

        // Set category
        hypothesis.category = HypothesisCategory::Behavioral;

        // Set confidence
        hypothesis.confidence = HypothesisConfidence::new(confidence);

        Some(hypothesis)
    }

    /// Decay confidence of old hypotheses
    pub async fn decay_hypotheses(&self) -> Result<usize, anyhow::Error> {
        tracing::debug!("Running hypothesis decay maintenance");

        // Get the hypothesis graph from the engine
        let graph = self.hypothesis_engine.get_graph();
        let mut graph_lock = graph
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        let mut decayed_count = 0;

        // Iterate over nodes and apply decay to their weight (proxy for confidence)
        for node in &mut graph_lock.nodes {
            // Apply time-based decay based on inactivity
            // Lower the weight over time if the hypothesis hasn't been updated
            if node.metadata.weight > 0.1 {
                // Apply small decay - reduce weight by 5% per decay cycle
                node.metadata.weight *= 0.95;
                decayed_count += 1;
            }
        }

        tracing::info!("Decayed {} hypotheses", decayed_count);
        Ok(decayed_count)
    }

    /// Score an experience based on outcome and context
    pub async fn score_experience(&self, experience: &Experience) -> f32 {
        let mut score = 0.5;

        // Factor in outcome
        match experience.outcome.kind {
            OutcomeKind::Success => score += 0.2,
            OutcomeKind::Partial => score += 0.1,
            OutcomeKind::Failure => score -= 0.1,
            OutcomeKind::Interrupted => score -= 0.05,
            _ => {}
        }

        // Factor in confidence
        score = score * 0.5 + experience.confidence * 0.5;

        // Factor in evidence count
        let evidence_factor = (experience.evidence_count as f32 * 0.02).min(0.2);
        score += evidence_factor;

        score.clamp(0.0, 1.0)
    }

    /// Generate reflection from experience
    ///
    /// Per Architecture §10:
    /// "Reflection asks: What happened? Why did it happen? Was the result expected? What should change?"
    pub async fn generate_reflection(
        &self,
        experience: &Experience,
    ) -> Result<crate::experience::reflection::Reflection, anyhow::Error> {
        let title = format!("Reflection on: {}", experience.title);

        let reflection = self
            .reflection_engine
            .generate_from_single(experience, title)
            .await?;

        // Publish ReflectionCompleted event
        let event = ExperienceEvent::reflection_completed(
            experience.id,
            Uuid::parse_str(&reflection.id).unwrap_or_default(),
        );
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish reflection event: {}", e);
        }

        Ok(reflection)
    }
}
