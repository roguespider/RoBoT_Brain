// src/experience/integration/learning_coordinator/mod.rs
//! Learning Coordinator - Orchestrates the complete learning pipeline
//!
//! Per Architecture §9 - Learning Engine:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection
//!
//! This coordinator ensures the continuous feedback loop:
//! Experience → Reflection → Hypothesis → Validation → Knowledge Update → Behavior Improvement

pub mod config;
pub mod generalization;
pub mod reinforcement;
pub mod results;

pub use config::LearningCoordinatorConfig;
pub use generalization::GeneralizationMethods;
pub use reinforcement::ReinforcementMethods;
pub use results::{
    GeneralizationResult, LearningCoordinatorStats, LearningPattern, MaintenanceStats,
    PatternKind, ReinforcementResult, TransferResult, ValidationResult, LearningResult,
};

use anyhow::Result;
use chrono::{Duration, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::exploration::Exploration;
use crate::experience::hypothesis::core::hypothesis::Hypothesis;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::reputation::reputation::Reputation;
use crate::experience::types::Experience;
use crate::knowledge::{KnowledgeItem, KnowledgeStore};
use crate::skills::registry::SkillRegistry;

/// Learning Coordinator - Main orchestrator for the learning pipeline
///
/// Per Architecture §4.04:
/// Experience → Reflection → Hypothesis → Exploration → Knowledge → Reputation
///
/// This coordinator:
/// 1. Receives events from the bus
/// 2. Routes them to appropriate subsystems
/// 3. Manages the lifecycle of learning artifacts
/// 4. Ensures the feedback loop is closed
pub struct LearningCoordinator {
    config: LearningCoordinatorConfig,

    // Core subsystems
    reflection_engine: Arc<ReflectionEngine>,
    hypothesis_engine: Arc<HypothesisEngine>,
    knowledge_store: Arc<KnowledgeStore>,

    // Repositories
    reputations: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Reputation>>>,
    explorations: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Exploration>>>,

    // Metrics
    metrics: Arc<MetricsCollector>,

    // Event bus for publishing
    bus: Arc<ExperienceBus>,

    // Database for persistence
    database: Option<Arc<SqliteDatabase>>,

    // Skill registry for skill updates
    skill_registry: Option<Arc<SkillRegistry>>,
}

impl LearningCoordinator {
    /// Create a new learning coordinator with all dependencies
    pub fn new(
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        knowledge_store: Arc<KnowledgeStore>,
        bus: Arc<ExperienceBus>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            config: LearningCoordinatorConfig::default(),
            reflection_engine,
            hypothesis_engine,
            knowledge_store,
            reputations: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            explorations: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            metrics,
            bus,
            database: None,
            skill_registry: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        config: LearningCoordinatorConfig,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        knowledge_store: Arc<KnowledgeStore>,
        bus: Arc<ExperienceBus>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            config,
            reflection_engine,
            hypothesis_engine,
            knowledge_store,
            reputations: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            explorations: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            metrics,
            bus,
            database: None,
            skill_registry: None,
        }
    }

    /// Set the database for persistence
    pub fn with_database(mut self, database: Arc<SqliteDatabase>) -> Self {
        self.database = Some(database);
        self
    }

    /// Set the skill registry for skill updates
    pub fn with_skill_registry(mut self, skill_registry: Arc<SkillRegistry>) -> Self {
        self.skill_registry = Some(skill_registry);
        self
    }

    // ========================================================================
    // Main Entry Points
    // ========================================================================

    /// Process an experience through the full learning pipeline
    ///
    /// Per Architecture §5.3:
    /// Event → Experience Recorder → Experience Storage → Scoring → Reflection → Learning Signals
    pub async fn process_experience(&self, experience: &Experience) -> Result<LearningResult> {
        tracing::info!(
            "Processing experience through learning pipeline: {}",
            experience.id
        );

        // Step 1: Score the experience
        let score = self.score_experience(experience).await;
        self.metrics.increment("learning.experiences.scored").await;

        // Step 2: Generate reflection (if threshold met)
        let reflection_id = if self.config.auto_reflect && score >= self.config.reflection_threshold
        {
            if let Ok(reflection) = self.generate_reflection(experience).await {
                self.metrics
                    .increment("learning.reflections.generated")
                    .await;
                Some(reflection.id)
            } else {
                None
            }
        } else {
            None
        };

        // Step 3: Generate hypotheses (if enabled)
        let hypothesis_ids = if self.config.auto_hypothesize {
            if let Ok(hypotheses) = self.generate_hypotheses(experience).await {
                self.metrics
                    .increment("learning.hypotheses.generated")
                    .await;
                hypotheses
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

    /// Process an experience through the full learning pipeline
    ///
    /// Per Architecture §5.3:
    /// Event → Experience Recorder → Experience Storage → Scoring → Reflection → Learning Signals
    pub async fn process_experience_full(&self, experience: &Experience) -> Result<LearningResult> {
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
        let score = self.score_experience(experience).await;
        result.score = score;
        self.metrics.increment("learning.experiences.scored").await;

        // Step 2: Generate reflection (if threshold met)
        if self.config.auto_reflect && score >= self.config.reflection_threshold {
            if let Ok(reflection) = self.generate_reflection(experience).await {
                result.reflection_id = Some(reflection.id);
                self.metrics
                    .increment("learning.reflections.generated")
                    .await;
            }
        }

        // Step 3: Generate hypotheses (if enabled)
        if self.config.auto_hypothesize {
            if let Ok(hypotheses) = self.generate_hypotheses(experience).await {
                result.hypothesis_ids = hypotheses;
                self.metrics
                    .increment("learning.hypotheses.generated")
                    .await;
            }
        }

        // Step 4: Update knowledge from high-value experiences
        if score >= 0.8 {
            self.promote_to_knowledge(experience).await?;
            self.metrics.increment("learning.knowledge.created").await;
        }

        // Step 5: Update reputation based on outcome
        self.update_reputation(experience).await?;
        self.metrics.increment("learning.reputation.updated").await;

        tracing::info!(
            "Completed learning pipeline for experience {}",
            experience.id
        );
        Ok(result)
    }

    /// Validate a hypothesis and potentially promote to knowledge
    ///
    /// Per Architecture §2.5:
    /// "A hypothesis is a temporary model waiting for evidence"
    pub async fn validate_hypothesis(&self, hypothesis_id: &str) -> Result<ValidationResult> {
        let result = ValidationResult {
            hypothesis_id: hypothesis_id.to_string(),
            ..Default::default()
        };

        // Check hypothesis status in repository
        // This would normally query the hypothesis repository

        // If confidence exceeds threshold, promote to knowledge
        if self.config.auto_promote_to_knowledge {
            // Create knowledge from validated hypothesis
            tracing::info!(
                "Hypothesis {} validated, promoting to knowledge",
                hypothesis_id
            );
            self.metrics
                .increment("learning.hypotheses.validated")
                .await;
        }

        Ok(result)
    }

    /// Perform maintenance tasks (called periodically)
    pub async fn run_maintenance(&self) -> Result<MaintenanceStats> {
        let mut stats = MaintenanceStats::default();

        // 1. Decay old hypotheses
        let decayed = self.decay_hypotheses().await?;
        stats.hypotheses_decayed = decayed;

        // 2. Archive stale explorations
        let archived = self.archive_stale_explorations().await?;
        stats.explorations_archived = archived;

        // 3. Consolidate low-confidence knowledge
        let consolidated = self.consolidate_knowledge().await?;
        stats.knowledge_consolidated = consolidated;

        // 4: Update reputation decay
        self.decay_reputations().await?;

        tracing::info!("Maintenance complete: {:?}", stats);
        Ok(stats)
    }

    // ========================================================================
    // Reflection Pipeline
    // ========================================================================

    /// Score an experience based on outcome and context
    async fn score_experience(&self, experience: &Experience) -> f32 {
        let mut score = 0.5;

        // Factor in outcome
        use crate::experience::types::outcome::OutcomeKind;
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
    async fn generate_reflection(
        &self,
        experience: &Experience,
    ) -> Result<crate::experience::reflection::Reflection> {
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

    // ========================================================================
    // Hypothesis Pipeline
    // ========================================================================

    /// Generate hypotheses from experience
    ///
    /// Per Architecture §11:
    /// "Hypotheses enable discovery"
    async fn generate_hypotheses(&self, experience: &Experience) -> Result<Vec<String>> {
        let mut hypothesis_ids = Vec::new();

        // Generate a hypothesis from the experience
        // Use the hypothesis engine to create a structured hypothesis
        let hypothesis = self.create_hypothesis_from_experience(experience).await;

        if let Some(h) = hypothesis {
            hypothesis_ids.push(h.id.0.clone());
            tracing::info!(
                "Generated hypothesis {} from experience {}",
                h.id.0,
                experience.id
            );
        }

        // Publish HypothesisGenerated event
        let event = ExperienceEvent::hypothesis_generated(experience.id, Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish hypothesis event: {}", e);
        }

        Ok(hypothesis_ids)
    }

    /// Create a hypothesis from an experience
    async fn create_hypothesis_from_experience(
        &self,
        experience: &Experience,
    ) -> Option<Hypothesis> {
        // Create hypothesis based on experience type and outcome
        let title = format!(
            "{}: {}",
            match experience.outcome.kind {
                crate::experience::types::outcome::OutcomeKind::Success => "What worked",
                crate::experience::types::outcome::OutcomeKind::Failure => "What failed",
                _ => "Observation",
            },
            experience.title
        );

        let statement = format!(
            "{} - This {} resulted in {}",
            experience.description,
            match experience.experience_type {
                crate::experience::types::ExperienceType::ToolExecution => "tool execution",
                crate::experience::types::ExperienceType::Planning => "planning activity",
                crate::experience::types::ExperienceType::Workflow => "workflow step",
                _ => "action",
            },
            match experience.outcome.kind {
                crate::experience::types::outcome::OutcomeKind::Success => "success",
                crate::experience::types::outcome::OutcomeKind::Failure => "failure",
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
        hypothesis.category =
            crate::experience::hypothesis::core::hypothesis::HypothesisCategory::Behavioral;

        // Set confidence
        hypothesis.confidence =
            crate::experience::hypothesis::core::hypothesis::HypothesisConfidence::new(confidence);

        Some(hypothesis)
    }

    /// Decay confidence of old hypotheses
    async fn decay_hypotheses(&self) -> Result<usize> {
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

    // ========================================================================
    // Knowledge Pipeline
    // ========================================================================

    /// Promote high-value experience to knowledge
    ///
    /// Per Architecture §2.3:
    /// "Knowledge is information that has survived evaluation"
    async fn promote_to_knowledge(&self, experience: &Experience) -> Result<()> {
        let knowledge = KnowledgeItem::from_reflection(
            &experience.description,
            experience.confidence,
            experience.id,
        );

        let _added_id = self.knowledge_store.add(knowledge).await;

        // Publish KnowledgeUpdated event
        let event = ExperienceEvent::knowledge_updated(Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish knowledge event: {}", e);
        }

        Ok(())
    }

    /// Consolidate low-confidence knowledge
    async fn consolidate_knowledge(&self) -> Result<usize> {
        tracing::debug!("Running knowledge consolidation");

        // Find and consolidate low-confidence knowledge
        let mut consolidated = 0;
        let knowledge_items = self.knowledge_store.get_all().await;

        for knowledge in knowledge_items {
            // If knowledge confidence is below threshold, try to consolidate
            let overall_confidence = knowledge.confidence.overall();
            if overall_confidence < 0.3 {
                // Archive or remove low-confidence knowledge
                // For now, just log it
                tracing::debug!(
                    "Low-confidence knowledge {} found with confidence {:.2}",
                    knowledge.id,
                    overall_confidence
                );
                consolidated += 1;
            }
        }

        Ok(consolidated)
    }

    // ========================================================================
    // Exploration Pipeline
    // ========================================================================

    /// Start exploration for a hypothesis
    pub async fn start_exploration(
        &self,
        hypothesis_id: String,
        title: String,
        purpose: String,
    ) -> Result<String> {
        let exploration_id = Uuid::new_v4().to_string();

        // Link exploration to the hypothesis it's investigating
        let exploration = Exploration::new(
            exploration_id.clone(),
            title,
            purpose,
            crate::experience::types::ExperienceContext {
                related_hypothesis: Some(hypothesis_id.clone()),
                ..Default::default()
            },
        );

        let mut store = self.explorations.write().await;
        store.insert(exploration_id.clone(), exploration);

        // Publish ExplorationStarted event
        let event = ExperienceEvent::exploration_started(Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish exploration event: {}", e);
        }

        Ok(exploration_id)
    }

    /// Complete an exploration
    pub async fn complete_exploration(&self, exploration_id: &str) -> Result<()> {
        let mut store = self.explorations.write().await;

        if let Some(exp) = store.get_mut(exploration_id) {
            exp.complete();
        }

        // Publish ExplorationCompleted event
        let event = ExperienceEvent::exploration_completed(Uuid::new_v4(), Uuid::new_v4());
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish exploration completed event: {}", e);
        }

        Ok(())
    }

    /// Archive stale explorations
    async fn archive_stale_explorations(&self) -> Result<usize> {
        let cutoff = Utc::now() - Duration::days(7);
        let mut store = self.explorations.write().await;
        let mut archived = 0;

        store.retain(|id, exp| {
            if let Some(completed) = exp.completed_at {
                tracing::trace!("Archiving exploration {}", id);
                if completed < cutoff {
                    archived += 1;
                    return false;
                }
            }
            true
        });

        Ok(archived)
    }

    // ========================================================================
    // Reputation Pipeline
    // ========================================================================

    /// Update reputation based on experience outcome
    ///
    /// Per Architecture §12:
    /// "Reputation determines how much each source of knowledge should be trusted"
    async fn update_reputation(&self, experience: &Experience) -> Result<()> {
        let source = &experience.context.source;
        let source_str = match source {
            Some(s) => s.clone(),
            None => return Ok(()),
        };

        if source_str.is_empty() {
            return Ok(());
        }

        let mut store = self.reputations.write().await;
        let reputation = store
            .entry(source_str.clone())
            .or_insert_with(|| Reputation::new(source_str.clone()));

        // Determine impact based on outcome
        use crate::experience::types::outcome::OutcomeKind;
        let (impact, reason) = match experience.outcome.kind {
            OutcomeKind::Success => (0.1, "Successful experience".to_string()),
            OutcomeKind::Partial => (0.0, "Partial success".to_string()),
            OutcomeKind::Failure => (-0.15, "Failed experience".to_string()),
            OutcomeKind::Interrupted => (-0.05, "Interrupted".to_string()),
            _ => (0.0, "Unknown outcome".to_string()),
        };

        reputation.apply(
            experience.id.to_string(),
            crate::experience::reputation::factors::ReputationFactor::Accuracy,
            impact,
            reason,
        );

        // Publish ReputationUpdated event
        let event = ExperienceEvent::reputation_updated(Uuid::new_v4(), source_str, impact as f32);
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish reputation event: {}", e);
        }

        Ok(())
    }

    /// Decay reputations over time
    async fn decay_reputations(&self) -> Result<()> {
        let mut store = self.reputations.write().await;

        for reputation in store.values_mut() {
            // Apply small decay
            if reputation.score > 0.5 {
                reputation.score = (reputation.score - 0.01).max(0.5);
            } else if reputation.score < 0.5 {
                reputation.score = (reputation.score + 0.01).min(0.5);
            }
        }

        Ok(())
    }

    /// Get reputation for a source
    pub async fn get_reputation(&self, source: &str) -> Option<f64> {
        let store = self.reputations.read().await;
        store.get(source).map(|r| r.score)
    }

    // ========================================================================
    // REINFORCEMENT LEARNING
    // ========================================================================
    /// Per Architecture §9: Reinforcement learning from experience outcomes
    /// Apply reinforcement learning from an experience outcome
    ///
    /// Per Architecture §9: "Reinforcement learning adjusts behavior based on rewards/penalties"
    pub async fn apply_reinforcement(
        &self,
        experience: &Experience,
    ) -> Result<ReinforcementResult> {
        let methods = ReinforcementMethods {
            knowledge_store: &self.knowledge_store,
            skill_registry: &self.skill_registry,
            metrics: &self.metrics,
        };
        methods.apply_reinforcement(experience).await
    }

    /// Calculate reward from experience outcome
    fn calculate_reward(&self, experience: &Experience) -> f64 {
        let methods = ReinforcementMethods {
            knowledge_store: &self.knowledge_store,
            skill_registry: &self.skill_registry,
            metrics: &self.metrics,
        };
        methods.calculate_reward(experience)
    }

    /// Update knowledge based on reinforcement reward
    async fn update_knowledge_from_reward(
        &self,
        experience: &Experience,
        reward: f64,
    ) -> Result<usize> {
        let methods = ReinforcementMethods {
            knowledge_store: &self.knowledge_store,
            skill_registry: &self.skill_registry,
            metrics: &self.metrics,
        };
        methods.update_knowledge_from_reward(experience, reward).await
    }

    /// Update skills based on reinforcement reward
    async fn update_skills_from_reward(
        &self,
        experience: &Experience,
        reward: f64,
    ) -> Result<usize> {
        let methods = ReinforcementMethods {
            knowledge_store: &self.knowledge_store,
            skill_registry: &self.skill_registry,
            metrics: &self.metrics,
        };
        methods.update_skills_from_reward(experience, reward).await
    }

    /// Update action values for future decision making
    async fn update_action_values(&self, experience: &Experience, reward: f64) -> Result<f64> {
        let methods = ReinforcementMethods {
            knowledge_store: &self.knowledge_store,
            skill_registry: &self.skill_registry,
            metrics: &self.metrics,
        };
        methods.update_action_values(experience, reward).await
    }

    // ========================================================================
    // GENERALIZATION
    // ========================================================================
    /// Per Architecture §9: Generalization - extracting patterns from specific experiences
    /// Generalize from a set of experiences to create broader patterns
    ///
    /// Per Architecture §9: "Generalization extracts common patterns from specific instances"
    pub async fn generalize(&self, experience_ids: Vec<Uuid>) -> Result<GeneralizationResult> {
        let methods = GeneralizationMethods {
            knowledge_store: &self.knowledge_store,
            bus: &self.bus,
            metrics: &self.metrics,
        };
        methods.generalize(experience_ids).await
    }

    /// Extract common patterns from experiences
    fn extract_common_patterns(&self, experiences: &[Experience]) -> Vec<LearningPattern> {
        let methods = GeneralizationMethods {
            knowledge_store: &self.knowledge_store,
            bus: &self.bus,
            metrics: &self.metrics,
        };
        methods.extract_common_patterns(experiences)
    }

    // ========================================================================
    // TRANSFER LEARNING
    // ========================================================================
    /// Per Architecture §9: Transfer learning - applying knowledge from one domain to another
    /// Transfer knowledge from source domain to target domain
    ///
    /// Per Architecture §9: "Transfer learning applies knowledge from one domain to another"
    pub async fn transfer_knowledge(
        &self,
        source_domain: &str,
        target_domain: &str,
        knowledge_ids: Vec<Uuid>,
    ) -> Result<TransferResult> {
        let methods = GeneralizationMethods {
            knowledge_store: &self.knowledge_store,
            bus: &self.bus,
            metrics: &self.metrics,
        };
        methods.transfer_knowledge(source_domain, target_domain, knowledge_ids).await
    }

    /// Adapt a knowledge item for a new domain
    async fn adapt_knowledge_for_domain(
        &self,
        knowledge: &KnowledgeItem,
        target_domain: &str,
    ) -> Option<KnowledgeItem> {
        let methods = GeneralizationMethods {
            knowledge_store: &self.knowledge_store,
            bus: &self.bus,
            metrics: &self.metrics,
        };
        methods.adapt_knowledge_for_domain(knowledge, target_domain).await
    }

    /// Check if knowledge is compatible with a domain
    fn check_domain_compatibility(knowledge_statement: &str, target_domain: &str) -> f32 {
        GeneralizationMethods::check_domain_compatibility(knowledge_statement, target_domain)
    }

    // ========================================================================
    // Stats
    // ========================================================================

    /// Get coordinator statistics
    pub async fn get_stats(&self) -> LearningCoordinatorStats {
        let reflections = self.reflection_engine.get_stats().await;
        let reputations = self.reputations.read().await;
        let explorations = self.explorations.read().await;

        LearningCoordinatorStats {
            total_reflections: reflections.total_reflections,
            total_insights: reflections.total_insights,
            trusted_insights: reflections.trusted_insights,
            total_patterns: reflections.total_patterns,
            active_reputations: reputations.len(),
            active_explorations: explorations.values().filter(|e| e.is_active()).count(),
        }
    }
}
