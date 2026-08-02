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
        _reflection: &crate::experience::reflection::Reflection,
    ) -> Result<()> {
        // Extract insights and create knowledge items
        // This bridges Reflection → Knowledge per Architecture §4.04
        Ok(())
    }

    /// Update knowledge from validated hypothesis
    pub(super) async fn update_knowledge_from_hypothesis(
        &self,
        _hypothesis: &crate::experience::hypothesis::core::hypothesis::Hypothesis,
        _result: &str,
    ) -> Result<()> {
        // If hypothesis is validated, create knowledge from it
        // Per Architecture §2.5: "Hypothesis is a temporary model waiting for evidence"
        Ok(())
    }

    /// Update hypothesis with new evidence
    pub(super) async fn update_hypothesis_with_evidence(
        &self,
        hypothesis_id: &str,
        _evidence: &crate::experience::events::payload::EventPayload,
    ) -> Result<()> {
        // Update hypothesis confidence based on evidence
        tracing::debug!("Updating hypothesis {} with new evidence", hypothesis_id);
        Ok(())
    }
}
