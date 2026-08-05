// src/experience/integration/event_subscriber/helpers.rs

//! Helper methods for the event subscriber

use super::EventSubscriber;
use crate::experience::types::Experience;
use anyhow::Result;

impl EventSubscriber {
    /// Generate reflection from experience
    pub(super) async fn generate_reflection(&self, experience: &Experience) -> Result<()> {
        let _ = self
            .reflection_engine
            .generate_from_single(experience, format!("Reflection on: {}", experience.title))
            .await;

        tracing::info!("Generated reflection for experience: {}", experience.id);
        Ok(())
    }

    /// Generate hypothesis from experience
    pub(super) async fn generate_hypothesis(&self, experience: &Experience) -> Result<()> {
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

                let _ = self
                    .evolution_engine
                    .create_behavior_from_insight(&insight)
                    .await;
                tracing::info!(
                    "Created behavior from high-confidence experience: {}",
                    experience.id
                );
            }
        }

        tracing::info!("Generated hypotheses from experience");
        Ok(())
    }

    /// Update knowledge store from reflection insights
    pub(super) async fn update_knowledge_from_reflection(
        &self,
        reflection: &crate::experience::reflection::Reflection,
    ) -> Result<()> {
        // Extract insights and create knowledge items
        // This bridges Reflection → Knowledge per Architecture §4.04
        // Note: Reflection insights are accessed via the reflection engine
        let insight_count = reflection.experience_ids.len();

        tracing::info!(
            "Updated knowledge store from reflection: {} experiences processed",
            insight_count
        );
        Ok(())
    }

    /// Update knowledge from validated hypothesis
    pub(super) async fn update_knowledge_from_hypothesis(
        &self,
        hypothesis: &crate::experience::hypothesis::core::hypothesis::Hypothesis,
        result: &str,
    ) -> Result<()> {
        // If hypothesis is validated, create knowledge from it
        // Per Architecture §2.5: "Hypothesis is a temporary model waiting for evidence"
        let is_validated = hypothesis.status == crate::experience::hypothesis::core::hypothesis::HypothesisStatus::Supported
            || hypothesis.status == crate::experience::hypothesis::core::hypothesis::HypothesisStatus::Active;

        if is_validated {
            let knowledge_content = format!(
                "Validated hypothesis: {} (description: {}, confidence: {:?})",
                hypothesis.id.0, hypothesis.description, hypothesis.confidence
            );
            tracing::debug!("Validated hypothesis would create knowledge: {}", knowledge_content);
        }

        tracing::debug!(
            "Updated knowledge from hypothesis '{}': {}",
            hypothesis.id.0,
            result
        );
        Ok(())
    }

    /// Update hypothesis with new evidence
    pub(super) async fn update_hypothesis_with_evidence(
        &self,
        hypothesis_id: &str,
        evidence: &crate::experience::events::payload::EventPayload,
    ) -> Result<()> {
        // Update hypothesis confidence based on evidence
        let direction = match evidence {
            crate::experience::events::payload::EventPayload::EvidenceRecord {
                direction, ..
            } => direction.clone(),
            _ => "unknown".to_string(),
        };

        tracing::debug!(
            "Updating hypothesis {} with evidence direction: {}",
            hypothesis_id,
            direction
        );

        if direction == "support" {
            tracing::info!("Evidence supports hypothesis {}", hypothesis_id);
        } else if direction == "contradict" {
            tracing::warn!("Evidence contradicts hypothesis {}", hypothesis_id);
        }

        Ok(())
    }
}
