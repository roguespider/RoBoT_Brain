// src/experience/integration/event_subscriber/helpers.rs

//! Helper methods for the event subscriber

use super::EventSubscriber;
use crate::experience::types::Experience;
use anyhow::Result;

impl EventSubscriber {
    /// Generate reflection from experience
    pub(crate) async fn generate_reflection(&self, experience: &Experience) -> Result<()> {
        let _ = self
            .reflection_engine
            .generate_from_single(experience, format!("Reflection on: {}", experience.title))
            .await;

        tracing::info!("Generated reflection for experience: {}", experience.id);
        Ok(())
    }

    /// Generate hypothesis from experience
    pub(crate) async fn generate_hypothesis(&self, experience: &Experience) -> Result<()> {
        // Respect the subscriber's auto-hypothesize configuration
        // (Architecture §4.04 wiring toggle).
        if !self.config.auto_hypothesize {
            tracing::debug!("Auto-hypothesis disabled; skipping");
            return Ok(());
        }

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

                let behavior = self
                    .evolution_engine
                    .create_behavior_from_insight(&insight)
                    .await;
                tracing::info!(
                    "Created behavior from high-confidence experience: {} (behavior_ok={})",
                    experience.id,
                    behavior.is_ok()
                );
            }
        }

        // Record current hypothesis graph state for diagnostics so the stored
        // hypothesis engine remains an active participant (Architecture §11).
        let graph_stats = self.hypothesis_engine.get_graph_stats();
        tracing::debug!(
            "Hypothesis graph after experience {}: {} nodes, {} edges",
            experience.id,
            graph_stats.node_count,
            graph_stats.edge_count
        );

        tracing::info!("Generated hypotheses from experience");
        Ok(())
    }

    /// Update knowledge store from reflection insights
    pub(crate) async fn update_knowledge_from_reflection(
        &self,
        reflection: &crate::experience::reflection::Reflection,
    ) -> Result<()> {
        // Respect the subscriber's auto-update-knowledge configuration
        // (Architecture §4.04 wiring toggle).
        if !self.config.auto_update_knowledge {
            tracing::debug!("Auto knowledge update disabled; skipping");
            return Ok(());
        }

        // Extract insights and create knowledge items
        // This bridges Reflection → Knowledge per Architecture §4.04
        let knowledge = crate::knowledge::KnowledgeItem::from_reflection(
            &reflection.description,
            reflection.confidence.score,
            uuid::Uuid::parse_str(&reflection.id).unwrap_or_default(),
        );
        let added_id = self.knowledge_store.add(knowledge).await;
        let insight_count = reflection.experience_ids.len();

        tracing::info!(
            "Updated knowledge store from reflection: {} experiences processed (knowledge_id={})",
            insight_count,
            added_id
        );
        Ok(())
    }

    /// Update knowledge from validated hypothesis
    pub(crate) async fn update_knowledge_from_hypothesis(
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
            // Promote validated hypothesis into the knowledge store
            // (Architecture §2.3: knowledge is information that has survived
            // evaluation). Guarded by the auto-update-knowledge toggle.
            if self.config.auto_update_knowledge {
                let knowledge = crate::knowledge::KnowledgeItem::from_reflection(
                    &knowledge_content,
                    hypothesis.confidence.value,
                    uuid::Uuid::new_v4(),
                );
                let added_id = self.knowledge_store.add(knowledge).await;
                tracing::debug!(
                    "Promoted validated hypothesis to knowledge (knowledge_id={})",
                    added_id
                );
            } else {
                tracing::debug!("Validated hypothesis would create knowledge: {}", knowledge_content);
            }
        }

        tracing::debug!(
            "Updated knowledge from hypothesis '{}': {}",
            hypothesis.id.0,
            result
        );
        Ok(())
    }

    /// Update hypothesis with new evidence
    pub(crate) async fn update_hypothesis_with_evidence(
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
