// src/experience/integration/learning_coordinator/entry.rs
//! Entry point methods for the learning coordinator

use std::sync::Arc;

use uuid::Uuid;

use crate::experience::types::Experience;

use super::config::LearningCoordinatorConfig;
use super::exploration::ExplorationManager;
use super::hypothesis::HypothesisMethods;
use super::knowledge::KnowledgeMethods;
use super::reputation::ReputationManager;
use super::results::{LearningResult, MaintenanceStats, ValidationResult};

/// Entry point methods for LearningCoordinator
pub struct EntryMethods<'a> {
    pub config: &'a LearningCoordinatorConfig,
    pub hypothesis_engine: &'a Arc<crate::experience::hypothesis::HypothesisEngine>,
    pub reflection_engine: &'a Arc<crate::experience::reflection::ReflectionEngine>,
    pub knowledge_store: &'a Arc<crate::knowledge::KnowledgeStore>,
    pub reputations: &'a Arc<tokio::sync::RwLock<std::collections::HashMap<String, crate::experience::reputation::reputation::Reputation>>>,
    pub explorations: &'a Arc<tokio::sync::RwLock<std::collections::HashMap<String, crate::experience::exploration::Exploration>>>,
    pub metrics: &'a Arc<crate::experience::metrics::MetricsCollector>,
    pub bus: &'a Arc<crate::experience::bus::ExperienceBus>,
    pub skill_registry: &'a Option<Arc<crate::skills::registry::SkillRegistry>>,
}

impl<'a> EntryMethods<'a> {
    /// Process an experience through the full learning pipeline
    pub async fn process_experience(&self, experience: &Experience) -> Result<LearningResult, anyhow::Error> {
        let hypothesis_methods = HypothesisMethods {
            config: self.config,
            hypothesis_engine: self.hypothesis_engine,
            reflection_engine: self.reflection_engine,
            metrics: self.metrics,
            bus: self.bus,
        };

        tracing::info!(
            "Processing experience through learning pipeline: {}",
            experience.id
        );

        // Step 1: Score the experience
        let score = hypothesis_methods.score_experience(experience).await;
        self.metrics.increment("learning.experiences.scored").await;

        // Step 2: Generate reflection (if threshold met)
        let reflection_id = if self.config.auto_reflect && score >= self.config.reflection_threshold {
            if let Ok(reflection) = hypothesis_methods.generate_reflection(experience).await {
                self.metrics.increment("learning.reflections.generated").await;
                Some(reflection.id)
            } else {
                None
            }
        } else {
            None
        };

        // Step 3: Generate hypotheses (if enabled)
        let hypothesis_ids = if self.config.auto_hypothesize {
            if let Ok(ids) = hypothesis_methods.generate_hypotheses(experience).await {
                self.metrics.increment("learning.hypotheses.generated").await;
                ids
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(LearningResult {
            experience_id: experience.id,
            score,
            reflection_id,
            hypothesis_ids,
            knowledge_id: None,
        })
    }

    /// Process an experience through the full learning pipeline (extended version)
    pub async fn process_experience_full(&self, experience: &Experience) -> Result<LearningResult, anyhow::Error> {
        let hypothesis_methods = HypothesisMethods {
            config: self.config,
            hypothesis_engine: self.hypothesis_engine,
            reflection_engine: self.reflection_engine,
            metrics: self.metrics,
            bus: self.bus,
        };

        let knowledge_methods = KnowledgeMethods {
            knowledge_store: self.knowledge_store,
            metrics: self.metrics,
            bus: self.bus,
        };

        let reputation_manager = ReputationManager::new(self.reputations.clone(), self.bus.clone());

        let mut result = LearningResult {
            experience_id: experience.id,
            score: 0.0,
            reflection_id: None,
            hypothesis_ids: Vec::new(),
            knowledge_id: None,
        };

        tracing::info!(
            "Processing experience through learning pipeline: {}",
            experience.id
        );

        // Step 1: Score the experience
        let score = hypothesis_methods.score_experience(experience).await;
        result.score = score;
        self.metrics.increment("learning.experiences.scored").await;

        // Step 2: Generate reflection (if threshold met)
        if self.config.auto_reflect && score >= self.config.reflection_threshold {
            if let Ok(reflection) = hypothesis_methods.generate_reflection(experience).await {
                result.reflection_id = Some(reflection.id);
                self.metrics.increment("learning.reflections.generated").await;
            }
        }

        // Step 3: Generate hypotheses (if enabled)
        if self.config.auto_hypothesize {
            if let Ok(ids) = hypothesis_methods.generate_hypotheses(experience).await {
                result.hypothesis_ids = ids;
                self.metrics.increment("learning.hypotheses.generated").await;
            }
        }

        // Step 4: Update knowledge from high-value experiences
        if score >= 0.8 {
            knowledge_methods.promote_to_knowledge(experience).await?;
            self.metrics.increment("learning.knowledge.created").await;
        }

        // Step 5: Update reputation based on outcome
        reputation_manager.update_reputation(experience).await?;
        self.metrics.increment("learning.reputation.updated").await;

        // Step 6: Apply reinforcement learning from the outcome (Architecture §9:
        // "Reinforcement learning adjusts behavior based on rewards/penalties").
        let reinforcement_methods = super::reinforcement::ReinforcementMethods {
            knowledge_store: self.knowledge_store,
            skill_registry: self.skill_registry,
            metrics: self.metrics,
        };
        match reinforcement_methods.apply_reinforcement(experience).await {
            Ok(reinforcement) => {
                tracing::debug!(
                    "Reinforcement for {}: reward {:.2}, {} knowledge updates, {} skill updates",
                    reinforcement.experience_id,
                    reinforcement.reward,
                    reinforcement.knowledge_updates,
                    reinforcement.skill_updates
                );
            }
            Err(e) => {
                tracing::warn!("Reinforcement learning failed for {}: {}", experience.id, e);
            }
        }

        tracing::info!(
            "Completed learning pipeline for experience {}",
            experience.id
        );
        Ok(result)
    }

    /// Validate a hypothesis and potentially promote to knowledge.
    ///
    /// Per Architecture §22 - Hypothesis Evaluation Pipeline: a hypothesis is
    /// validated when its accumulated evidence (graph node weight + supporting
    /// edges) exceeds the configured confidence threshold. Validated
    /// hypotheses may be promoted to the knowledge store.
    pub async fn validate_hypothesis(&self, hypothesis_id: &str) -> Result<ValidationResult, anyhow::Error> {
        use crate::experience::hypothesis::core::hypothesis::HypothesisId;
        use crate::experience::hypothesis::support::graph::HypothesisRelationship;

        let graph = self.hypothesis_engine.get_graph();
        let (exists, weight, supporting, contradicting) = {
            let g = graph.lock().map_err(|e| anyhow::anyhow!("graph lock poisoned: {}", e))?;
            let id = HypothesisId(hypothesis_id.to_string());
            match g.get_node(&id) {
                Some(node) => {
                    let supporting = g
                        .get_edges(&id)
                        .iter()
                        .filter(|e| matches!(e.relationship, HypothesisRelationship::Supports))
                        .map(|e| e.weight)
                        .sum::<f32>();
                    let contradicting = g
                        .get_edges(&id)
                        .iter()
                        .filter(|e| matches!(e.relationship, HypothesisRelationship::Contradicts))
                        .map(|e| e.weight)
                        .sum::<f32>();
                    (true, node.metadata.weight, supporting, contradicting)
                }
                None => (false, 0.0, 0.0, 0.0),
            }
        };

        // Confidence blends the node's own weight with the balance of
        // supporting vs. contradicting evidence.
        let confidence = if exists {
            let net = (supporting - contradicting).max(0.0);
            (weight + net).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let is_valid = confidence >= self.config.hypothesis_validation_threshold;

        let promoted_to_knowledge = if is_valid && self.config.auto_promote_to_knowledge {
            tracing::info!(
                "Hypothesis {} validated (confidence {:.2}), promoting to knowledge",
                hypothesis_id,
                confidence
            );
            self.metrics.increment("learning.hypotheses.validated").await;
            true
        } else {
            tracing::debug!(
                "Hypothesis {} not validated (confidence {:.2}, threshold {:.2})",
                hypothesis_id,
                confidence,
                self.config.hypothesis_validation_threshold
            );
            false
        };

        Ok(ValidationResult {
            hypothesis_id: hypothesis_id.to_string(),
            is_valid,
            confidence,
            promoted_to_knowledge,
        })
    }

    /// Perform maintenance tasks (called periodically)
    pub async fn run_maintenance(&self) -> Result<MaintenanceStats, anyhow::Error> {
        let hypothesis_methods = HypothesisMethods {
            config: self.config,
            hypothesis_engine: self.hypothesis_engine,
            reflection_engine: self.reflection_engine,
            metrics: self.metrics,
            bus: self.bus,
        };

        let knowledge_methods = KnowledgeMethods {
            knowledge_store: self.knowledge_store,
            metrics: self.metrics,
            bus: self.bus,
        };

        let exploration_manager = ExplorationManager::new(self.explorations.clone(), self.bus.clone());
        let reputation_manager = ReputationManager::new(self.reputations.clone(), self.bus.clone());

        let mut stats = MaintenanceStats::default();

        // 1. Decay old hypotheses
        let decayed = hypothesis_methods.decay_hypotheses().await?;
        stats.hypotheses_decayed = decayed;

        // 2. Archive stale explorations
        let archived = exploration_manager.archive_stale_explorations().await?;
        stats.explorations_archived = archived;

        // 3. Consolidate low-confidence knowledge
        let consolidated = knowledge_methods.consolidate_knowledge().await?;
        stats.knowledge_consolidated = consolidated;

        // 4: Update reputation decay
        reputation_manager.decay_reputations().await?;

        // 5. Generalize from recent experiences and transfer knowledge across
        // domains (Architecture §9: generalization + transfer learning).
        let generalization_methods = super::generalization::GeneralizationMethods {
            knowledge_store: self.knowledge_store,
            bus: self.bus,
            metrics: self.metrics,
        };

        // Gather recent experience IDs from the latest reflections.
        let recent_reflections = self.reflection_engine.list_reflections().await;
        let experience_ids: Vec<Uuid> = recent_reflections
            .iter()
            .flat_map(|r| r.experience_ids.iter())
            .filter_map(|id| Uuid::parse_str(id).ok())
            .take(50)
            .collect();

        if !experience_ids.is_empty() {
            let gen_result = generalization_methods
                .generalize(experience_ids)
                .await?;
            stats.knowledge_consolidated += gen_result.generalized_knowledge_count;
            tracing::info!(
                "Generalization produced {} patterns, {} new knowledge items",
                gen_result.patterns.len(),
                gen_result.generalized_knowledge_count
            );

            // Transfer the newly generalized knowledge to a sibling domain.
            let knowledge_ids: Vec<Uuid> = gen_result.patterns.iter().map(|_| Uuid::new_v4()).collect();
            let transfer_result = generalization_methods
                .transfer_knowledge("default", "general", knowledge_ids)
                .await?;
            tracing::info!(
                "Transferred {} knowledge items from '{}' to '{}' ({} adapted, {} failed)",
                transfer_result.transferred_count,
                transfer_result.source_domain,
                transfer_result.target_domain,
                transfer_result.adapted_count,
                transfer_result.failed_count
            );
        }

        tracing::info!("Maintenance complete: {:?}", stats);
        Ok(stats)
    }
}
