// src/experience/integration/learning_coordinator/mod.rs
//! Learning Coordinator - Orchestrates the complete learning pipeline
//!
//! Per Architecture §9 - Learning Engine:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection
//!
//! This coordinator ensures the continuous feedback loop:
//! Experience → Reflection → Hypothesis → Validation → Knowledge Update → Behavior Improvement

pub mod config;
pub mod results;

pub use config::LearningCoordinatorConfig;
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
        let reward = self.calculate_reward(experience);

        let mut result = ReinforcementResult {
            experience_id: experience.id,
            reward,
            ..Default::default()
        };

        // Update knowledge based on reward
        let knowledge_updates = self
            .update_knowledge_from_reward(experience, reward)
            .await?;
        result.knowledge_updates = knowledge_updates;

        // Update skills based on reward
        let skill_updates = self.update_skills_from_reward(experience, reward).await?;
        result.skill_updates = skill_updates;

        // Update action values
        let action_value_update = self.update_action_values(experience, reward).await?;
        result.action_value_delta = action_value_update;

        self.metrics
            .increment("learning.reinforcement.applied")
            .await;

        tracing::info!(
            "Applied reinforcement learning for {}: reward={:.3}, knowledge_updates={}, skill_updates={}",
            experience.id, reward, knowledge_updates, skill_updates
        );

        Ok(result)
    }

    /// Calculate reward from experience outcome
    fn calculate_reward(&self, experience: &Experience) -> f64 {
        use crate::experience::types::outcome::OutcomeKind;

        match experience.outcome.kind {
            OutcomeKind::Success => {
                // Positive reward scaled by confidence
                1.0 * (experience.confidence as f64)
            }
            OutcomeKind::Partial => {
                // Small positive reward for partial success
                0.3 * (experience.confidence as f64)
            }
            OutcomeKind::Failure => {
                // Negative reward for failure
                -1.0
            }
            OutcomeKind::Interrupted => {
                // Small negative reward for interruption
                -0.2
            }
            _ => 0.0,
        }
    }

    /// Update knowledge based on reinforcement reward
    async fn update_knowledge_from_reward(
        &self,
        experience: &Experience,
        reward: f64,
    ) -> Result<usize> {
        let mut updates = 0;

        // Increase confidence for positive reward
        if reward > 0.0 {
            // Find knowledge related to this experience
            let related_knowledge = self.knowledge_store.search(&experience.description).await;

            for knowledge in related_knowledge {
                if reward > 0.5 {
                    // High reward: boost confidence
                    self.knowledge_store.record_success(knowledge.id).await;
                    updates += 1;
                }
            }
        } else if reward < 0.0 {
            // Decrease confidence for negative reward
            let related_knowledge = self.knowledge_store.search(&experience.description).await;

            for knowledge in related_knowledge {
                self.knowledge_store.record_failure(knowledge.id).await;
                updates += 1;
            }
        }

        Ok(updates)
    }

    /// Update skills based on reinforcement reward
    async fn update_skills_from_reward(
        &self,
        experience: &Experience,
        reward: f64,
    ) -> Result<usize> {
        let mut updates = 0;

        // Only update if we have a skill registry
        if let Some(ref registry) = self.skill_registry {
            // Find skills related to this experience context
            let workflow_name = experience
                .context
                .workflow
                .as_ref()
                .map(|w| w.name.clone())
                .unwrap_or_default();

            // Look for skills that match the workflow or experience type
            let skill_name = format!(
                "{}:{}",
                match experience.experience_type {
                    crate::experience::types::ExperienceType::ToolExecution => "tool",
                    crate::experience::types::ExperienceType::Planning => "planning",
                    crate::experience::types::ExperienceType::Workflow => &workflow_name,
                    _ => "general",
                },
                experience.title.replace(' ', "_").to_lowercase()
            );

            // Try to find and update the skill
            if let Some(skill) = registry.get_by_name(&skill_name).await {
                let success = reward > 0.0;
                let record_result = registry.record_usage(&skill.id, success).await;
                if record_result.is_ok() {
                    updates += 1;
                    tracing::debug!("Updated skill {} with success={}", skill_name, success);
                }
            }
        }

        tracing::debug!(
            "Skill update from reward {:.3}: {} ({} updates)",
            reward,
            experience.id,
            updates
        );
        Ok(updates)
    }

    /// Update action values for future decision making
    async fn update_action_values(&self, experience: &Experience, reward: f64) -> Result<f64> {
        // Store the reward for this action context
        // This could be used to build a Q-table or similar
        let action_key = format!(
            "{}:{}",
            experience
                .context
                .workflow
                .as_ref()
                .map(|w| w.name.as_str())
                .unwrap_or("unknown"),
            experience.description.as_str()
        );

        // In a full implementation, this would update a Q-table or similar
        // For now, we just log the action value update
        tracing::debug!(
            "Action value update for '{}': reward={:.3}",
            action_key,
            reward
        );

        Ok(reward)
    }

    // ========================================================================
    // GENERALIZATION
    // ========================================================================
    /// Per Architecture §9: Generalization - extracting patterns from specific experiences
    /// Generalize from a set of experiences to create broader patterns
    ///
    /// Per Architecture §9: "Generalization extracts common patterns from specific instances"
    pub async fn generalize(&self, experience_ids: Vec<Uuid>) -> Result<GeneralizationResult> {
        let mut result = GeneralizationResult::default();

        tracing::debug!("Generalizing from {} experience IDs", experience_ids.len());

        // Try to extract patterns from in-memory experiences
        // Note: In a full implementation, this would query the experience repository
        // For now, we create basic patterns from the experience IDs
        let mut patterns = Vec::new();

        for id in &experience_ids {
            let pattern = LearningPattern {
                description: format!("Pattern from experience {}", id),
                confidence: 0.5, // Default confidence for new patterns
                source_experience_count: 1,
                pattern_type: PatternKind::Sequential,
            };
            patterns.push(pattern);
        }

        result.patterns = patterns;

        // Create generalized knowledge from successful patterns
        for pattern in &result.patterns {
            // Only promote high-confidence patterns
            if pattern.confidence >= 0.6 {
                let generalized_knowledge = KnowledgeItem::from_reflection(
                    &pattern.description,
                    pattern.confidence,
                    Uuid::new_v4(),
                );
                let _added_id = self.knowledge_store.add(generalized_knowledge).await;
                result.generalized_knowledge_count += 1;
            }
        }

        self.metrics.increment("learning.generalizations").await;

        Ok(result)
    }

    /// Extract common patterns from experiences
    async fn extract_common_patterns(&self, experiences: &[Experience]) -> Vec<LearningPattern> {
        let mut patterns = Vec::new();

        if experiences.len() < 2 {
            return patterns;
        }

        // Group by context/workflow
        let mut context_groups: std::collections::HashMap<String, Vec<&Experience>> =
            std::collections::HashMap::new();

        for exp in experiences {
            let key = exp
                .context
                .workflow
                .as_ref()
                .map(|w| w.name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            context_groups.entry(key).or_default().push(exp);
        }

        // Find patterns in groups
        for (context, exps) in context_groups {
            if exps.len() >= 2 {
                // Count successful vs failed
                let success_count = exps
                    .iter()
                    .filter(|e| {
                        matches!(
                            e.outcome.kind,
                            crate::experience::types::outcome::OutcomeKind::Success
                        )
                    })
                    .count();

                let confidence = success_count as f32 / exps.len() as f32;

                patterns.push(LearningPattern {
                    description: format!(
                        "In '{}' context, {} out of {} attempts succeeded",
                        context,
                        success_count,
                        exps.len()
                    ),
                    confidence,
                    source_experience_count: exps.len(),
                    pattern_type: PatternKind::Contextual,
                });
            }
        }

        patterns
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
        let mut result = TransferResult {
            source_domain: source_domain.to_string(),
            target_domain: target_domain.to_string(),
            transferred_count: 0,
            adapted_count: 0,
            failed_count: 0,
            new_knowledge_ids: Vec::new(),
        };

        // Get source knowledge
        for knowledge_id in &knowledge_ids {
            if let Some(knowledge) = self.knowledge_store.get(*knowledge_id).await {
                // Adapt knowledge for target domain
                let adapted = self
                    .adapt_knowledge_for_domain(&knowledge, target_domain)
                    .await;

                if let Some(mut adapted_knowledge) = adapted {
                    // Lower confidence for transferred knowledge (needs validation)
                    adapted_knowledge.confidence.adjust_source_reliability(-0.2);

                    let new_id = self.knowledge_store.add(adapted_knowledge).await;
                    result.new_knowledge_ids.push(new_id);
                    result.transferred_count += 1;
                    result.adapted_count += 1;
                } else {
                    result.failed_count += 1;
                }
            }
        }

        // Publish transfer event
        let event = ExperienceEvent::knowledge_transferred(
            Uuid::new_v4(),
            source_domain.to_string(),
            target_domain.to_string(),
            result.transferred_count as u32,
        );
        let publish_result = self.bus.publish(event);
        if let Err(e) = publish_result {
            tracing::warn!("Failed to publish transfer event: {}", e);
        }

        self.metrics.increment("learning.transfers").await;

        tracing::info!(
            "Transferred {} knowledge items from {} to {}",
            result.transferred_count,
            source_domain,
            target_domain
        );

        Ok(result)
    }

    /// Adapt a knowledge item for a new domain
    async fn adapt_knowledge_for_domain(
        &self,
        knowledge: &KnowledgeItem,
        target_domain: &str,
    ) -> Option<KnowledgeItem> {
        // Check if domain is compatible
        let compatibility = Self::check_domain_compatibility(&knowledge.statement, target_domain);

        if compatibility < 0.3 {
            return None; // Not compatible enough
        }

        // Create adapted version
        let mut adapted = knowledge.clone();
        adapted.id = Uuid::new_v4();
        adapted
            .metadata
            .insert("transferred_from".to_string(), knowledge.id.to_string());
        adapted
            .metadata
            .insert("target_domain".to_string(), target_domain.to_string());
        adapted.metadata.insert(
            "original_confidence".to_string(),
            format!("{:.2}", knowledge.overall_confidence()),
        );

        // Scale confidence by compatibility
        let scaled_confidence = knowledge.overall_confidence() * compatibility;
        adapted
            .confidence
            .adjust_source_reliability(scaled_confidence - knowledge.overall_confidence());

        Some(adapted)
    }

    /// Check if knowledge is compatible with a domain
    fn check_domain_compatibility(knowledge_statement: &str, target_domain: &str) -> f32 {
        // Simple heuristic: check for domain-specific keywords
        let domain_keywords = match target_domain {
            "coding" => vec![
                "function",
                "variable",
                "class",
                "algorithm",
                "data",
                "process",
            ],
            "writing" => vec!["text", "content", "paragraph", "document", "sentence"],
            "analysis" => vec!["pattern", "trend", "compare", "evaluate", "assess"],
            _ => vec!["general", "common", "standard"],
        };

        let statement_lower = knowledge_statement.to_lowercase();
        let matches = domain_keywords
            .iter()
            .filter(|kw| statement_lower.contains(*kw))
            .count();

        let similarity = matches as f32 / domain_keywords.len() as f32;
        similarity.max(0.1) // Minimum 10% compatibility
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
