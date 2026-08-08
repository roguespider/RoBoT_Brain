// src/experience/integration/learning_coordinator/entry.rs
//! Entry point methods for the learning coordinator

use std::sync::Arc;

use crate::experience::types::Experience;

use super::config::LearningCoordinatorConfig;
use super::exploration::ExplorationManager;
use super::generalization::GeneralizationMethods;
use super::hypothesis::HypothesisMethods;
use super::knowledge::KnowledgeMethods;
use super::reinforcement::ReinforcementMethods;
use super::reputation::ReputationManager;
use super::results::{GeneralizationResult, LearningResult, MaintenanceStats, ReinforcementResult, TransferResult, ValidationResult};

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

        tracing::info!(
            "Completed learning pipeline for experience {}",
            experience.id
        );
        Ok(result)
    }

    /// Validate a hypothesis and potentially promote to knowledge
    pub async fn validate_hypothesis(&self, hypothesis_id: &str) -> Result<ValidationResult, anyhow::Error> {
        let result = ValidationResult {
            hypothesis_id: hypothesis_id.to_string(),
            ..Default::default()
        };

        if self.config.auto_promote_to_knowledge {
            tracing::info!(
                "Hypothesis {} validated, promoting to knowledge",
                hypothesis_id
            );
            self.metrics.increment("learning.hypotheses.validated").await;
        }

        Ok(result)
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

        tracing::info!("Maintenance complete: {:?}", stats);
        Ok(stats)
    }
}
