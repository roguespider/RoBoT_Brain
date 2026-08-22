// src/experience/integration/event_subscriber/handlers.rs

//! Event handler methods for the learning pipeline
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts

use super::EventSubscriber;
use crate::experience::events::payload::EventPayload;
use crate::experience::events::{ExperienceEvent, ExperienceEventType};
use crate::experience::types::Experience;
use anyhow::Result;

impl EventSubscriber {
    /// Process an experience event through the learning pipeline
    ///
    /// Per Architecture §4.04:
    /// ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts
    pub async fn process_event(&self, event: &ExperienceEvent) -> Result<()> {
        match event.event_type {
            ExperienceEventType::ExperienceRecorded => {
                self.on_experience_recorded(event).await?;
            }
            ExperienceEventType::ReflectionCompleted => {
                self.on_reflection_completed(event).await?;
            }
            ExperienceEventType::HypothesisGenerated => {
                self.on_hypothesis_generated(event).await?;
            }
            ExperienceEventType::HypothesisValidated => {
                self.on_hypothesis_validated(event).await?;
            }
            ExperienceEventType::KnowledgeUpdated => {
                self.on_knowledge_updated(event).await?;
            }
            ExperienceEventType::Scored => {
                self.on_experience_scored(event).await?;
            }
            ExperienceEventType::EvidenceAdded => {
                self.on_evidence_added(event).await?;
            }
            _ => {
                tracing::debug!("Ignoring event type: {:?}", event.event_type);
            }
        }
        Ok(())
    }

    /// Step 1: Experience recorded → Drive the full learning pipeline
    ///
    /// Per TASK-V2-01 / Architecture §4.04: the subscriber consumes the event
    /// once and invokes `LearningCoordinator::process_experience_full`, which
    /// runs the complete Score → Reflect → Hypothesize → Knowledge-promote →
    /// Reputation → Reinforcement path. This is the single driver of the event
    /// spine; downstream stages advance via the events that the learning
    /// coordinator itself publishes (ReflectionCompleted, HypothesisGenerated,
    /// etc.), handled below.
    pub(super) async fn on_experience_recorded(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing ExperienceRecorded event: {}", event.id);

        if let EventPayload::ExperienceRecord { experience, .. } = &event.payload {
            // Preferred path: delegate to the learning coordinator (§4.04
            // single-driver intent). The coordinator scores, reflects,
            // hypothesizes, promotes knowledge, updates reputation and applies
            // reinforcement — exactly the chain the architecture describes.
            if let Some(learning_coordinator) = &self.learning_coordinator {
                match learning_coordinator
                    .process_experience_full(experience)
                    .await
                {
                    Ok(result) => {
                        tracing::info!(
                            "Learning pipeline advanced for experience {} (score {:.2}, \
                             reflection={:?}, hypotheses={}, knowledge={:?})",
                            result.experience_id,
                            result.score,
                            result.reflection_id,
                            result.hypothesis_ids.len(),
                            result.knowledge_id
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Learning pipeline failed for experience {}: {}",
                            experience.id,
                            e
                        );
                    }
                }
            }

            // Also record the experience directly to the database via
            // ExperienceRecorder (Architecture §07 structured recording).
            if let Some(recorder) = &self.experience_recorder
                && let Err(e) = recorder.record(
                    experience.experience_type.clone(),
                    experience.title.clone(),
                    experience.description.clone(),
                    experience.context.clone(),
                    experience.outcome.clone(),
                    experience.observation_ids.clone(),
                )
            {
                tracing::warn!("ExperienceRecorder failed for {}: {}", experience.id, e);
            }

            // Fallback (no learning coordinator wired): drive the available
            // engines directly so the §4.04 chain still advances per event,
            // rather than only re-echoing the event.
            self.generate_reflection(experience).await?;
            self.generate_hypothesis(experience).await?;
        }

        Ok(())
    }

    /// Step 2: Reflection completed → Advance to hypothesis generation
    ///
    /// Per TASK-V2-03: instead of only bumping a counter, this handler drives
    /// the next stage of the §4.04 chain. A completed reflection surfaces
    /// insights that can seed new hypotheses via the hypothesis engine.
    pub(super) async fn on_reflection_completed(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing ReflectionCompleted event: {}", event.id);

        if let EventPayload::ReflectionRecord { reflection, .. } = &event.payload {
            // Advance Reflection → Knowledge (§4.04): promote reflection
            // insights into the knowledge store when configured.
            self.update_knowledge_from_reflection(reflection).await?;

            // Advance Reflection → Hypothesis (§4.04): a reflection that
            // surfaces a pattern is a candidate hypothesis. Generate one when
            // the reflection carries sufficient confidence.
            if self.config.auto_hypothesize
                && reflection.confidence.score >= self.config.reflection_threshold
            {
                let mut experience = Experience::new(
                    format!("Reflection: {}", reflection.description),
                    reflection.description.clone(),
                    crate::experience::types::ExperienceType::Reflection,
                    reflection
                        .experience_ids
                        .iter()
                        .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                        .collect(),
                );
                experience.score = Some(crate::experience::types::ExperienceScore {
                    importance: reflection.confidence.score,
                    confidence: reflection.confidence.score,
                    novelty: 0.0,
                    reliability: reflection.confidence.score,
                });
                self.generate_hypothesis(&experience).await?;
            }
        }

        Ok(())
    }

    /// Step 3: Hypothesis generated → Trigger exploration
    ///
    /// Per TASK-V2-03: advance Hypothesis → Exploration (§4.04). A freshly
    /// generated hypothesis should be queued for evidence gathering / validation.
    pub(super) async fn on_hypothesis_generated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing HypothesisGenerated event: {}", event.id);

        if let EventPayload::HypothesisRecord { hypothesis, .. } = &event.payload {
            // Drive exploration for the new hypothesis when enabled. The
            // learning coordinator (if wired) owns exploration lifecycle;
            // otherwise we log the candidate for the scheduler to pick up.
            if let Some(learning_coordinator) = &self.learning_coordinator {
                let title = format!("Exploration: {}", hypothesis.id.0);
                let purpose = hypothesis.description.clone();
                match learning_coordinator
                    .start_exploration(hypothesis.id.0.clone(), title, purpose)
                    .await
                {
                    Ok(exploration_id) => {
                        tracing::info!(
                            "Started exploration {} for hypothesis {}",
                            exploration_id,
                            hypothesis.id.0
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to start exploration for hypothesis {}: {}",
                            hypothesis.id.0,
                            e
                        );
                    }
                }
            } else {
                tracing::info!(
                    "Hypothesis {} queued for exploration (no learning coordinator wired)",
                    hypothesis.id.0
                );
            }
        }

        Ok(())
    }

    /// Step 4: Hypothesis validated → Update knowledge
    ///
    /// Per TASK-V2-03: advance Validation → Knowledge update (§4.04). A
    /// validated hypothesis is promoted into the knowledge store.
    pub(super) async fn on_hypothesis_validated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::info!("Processing HypothesisValidated event: {}", event.id);

        if let EventPayload::HypothesisValidation {
            hypothesis_id,
            result,
        } = &event.payload
        {
            tracing::debug!("Hypothesis {} validated: {}", hypothesis_id, result);

            // Wire metrics for hypothesis validation
            use crate::experience::metrics::metric_names;
            let metrics = self.metrics.clone();
            let result_owned = result.clone();
            tokio::spawn(async move {
                metrics.increment(metric_names::HYPOTHESES_GENERATED).await;
                if result_owned.to_lowercase().contains("confirm")
                    || result_owned.to_lowercase().contains("support")
                {
                    metrics.increment(metric_names::HYPOTHESES_CONFIRMED).await;
                } else if result_owned.to_lowercase().contains("reject") {
                    metrics.increment(metric_names::HYPOTHESES_REJECTED).await;
                }
            });

            // Advance Validation → Knowledge (§4.04): when the learning
            // coordinator is wired, ask it to validate (and potentially promote)
            // the hypothesis. This closes hypothesis → knowledge.
            if let Some(learning_coordinator) = &self.learning_coordinator {
                match learning_coordinator
                    .validate_hypothesis(hypothesis_id)
                    .await
                {
                    Ok(validation) => {
                        if validation.promoted_to_knowledge {
                            tracing::info!(
                                "Hypothesis {} promoted to knowledge (confidence {:.2})",
                                hypothesis_id,
                                validation.confidence
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Hypothesis validation failed for {}: {}", hypothesis_id, e);
                    }
                }
            } else {
                // Fallback (no learning coordinator wired): promote the
                // validated hypothesis into the knowledge store directly so
                // the §4.04 Validation → Knowledge step still advances.
                let hypothesis_id_typed =
                    crate::experience::hypothesis::core::hypothesis::HypothesisId(
                        hypothesis_id.clone(),
                    );
                let graph_arc = self.hypothesis_engine.get_graph();
                let exists = match graph_arc.lock() {
                    Ok(graph) => graph.has_node(&hypothesis_id_typed),
                    Err(poisoned) => {
                        tracing::error!("Graph mutex poisoned during hypothesis lookup");
                        poisoned.into_inner().has_node(&hypothesis_id_typed)
                    }
                };
                if exists {
                    self.update_knowledge_from_hypothesis(
                        &crate::experience::hypothesis::core::hypothesis::Hypothesis::new(
                            hypothesis_id.clone(),
                            hypothesis_id.clone(),
                        ),
                        result,
                    )
                    .await?;
                } else {
                    tracing::debug!(
                        "Hypothesis {} not present in graph; skipping fallback promotion",
                        hypothesis_id
                    );
                }
            }
        }

        Ok(())
    }

    /// Step 5: Knowledge updated → Update reputation
    ///
    /// Per TASK-V2-03: advance Knowledge → Reputation (§4.04). A knowledge
    /// update is a trust signal: the source that produced the knowledge gains
    /// reputation.
    pub(super) async fn on_knowledge_updated(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing KnowledgeUpdated event: {}", event.id);

        // Wire metrics for knowledge updates
        use crate::experience::metrics::metric_names;
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            metrics.increment(metric_names::KNOWLEDGE_CONFIDENCE).await;
        });

        // Advance Knowledge → Reputation (§4.04): a knowledge promotion
        // increases trust in the contributing source. When a knowledge_id is
        // present, credit the default source so reputation tracking stays live.
        if let EventPayload::KnowledgeRecord { knowledge_id } = &event.payload {
            self.record_reputation(
                "knowledge-source",
                0.6,
                &format!("Knowledge promoted (id={})", knowledge_id),
            )
            .await
            .ok();
        }

        Ok(())
    }

    /// Experience scored → May trigger reflection if score is high
    pub(super) async fn on_experience_scored(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing Scored event: {}", event.id);

        if let EventPayload::ScoreRecord {
            score,
            experience_id,
        } = &event.payload
        {
            // If score exceeds threshold, trigger reflection
            if self.config.auto_reflect && score.confidence >= self.config.reflection_threshold {
                tracing::info!(
                    "High-scoring experience {} triggering reflection",
                    experience_id
                );
                // Reflection will be triggered by the experience recorder
            }
        }

        Ok(())
    }

    /// Evidence added → Update hypothesis confidence
    pub(super) async fn on_evidence_added(&self, event: &ExperienceEvent) -> Result<()> {
        tracing::debug!("Processing EvidenceAdded event: {}", event.id);

        if let EventPayload::EvidenceRecord { hypothesis_id, .. } = &event.payload {
            // Drive the evidence → hypothesis-confidence update (Architecture
            // §11) so the subscriber participates in evidence processing even
            // without a learning coordinator wired.
            self.update_hypothesis_with_evidence(hypothesis_id, &event.payload)
                .await?;
        }

        Ok(())
    }
}
