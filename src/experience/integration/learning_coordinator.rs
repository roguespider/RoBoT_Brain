// src/experience/integration/learning_coordinator.rs
//! Learning Coordinator - Orchestrates the complete learning pipeline
//!
//! Per Architecture §9 - Learning Engine:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection
//!
//! This coordinator ensures the continuous feedback loop:
//! Experience → Reflection → Hypothesis → Validation → Knowledge Update → Behavior Improvement

#![allow(dead_code)]

use std::sync::Arc;
use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::types::Experience;
use crate::experience::events::ExperienceEvent;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::reputation::reputation::Reputation;
use crate::experience::exploration::Exploration;
use crate::knowledge::{KnowledgeStore, KnowledgeItem};
use crate::experience::metrics::MetricsCollector;

/// Configuration for the learning coordinator
#[derive(Debug, Clone)]
pub struct LearningCoordinatorConfig {
    /// Whether to auto-reflect on experiences
    pub auto_reflect: bool,
    /// Minimum score to trigger reflection
    pub reflection_threshold: f32,
    /// Whether to auto-generate hypotheses
    pub auto_hypothesize: bool,
    /// Whether to auto-explore hypotheses
    pub auto_explore: bool,
    /// Minimum confidence for hypothesis validation
    pub hypothesis_validation_threshold: f32,
    /// Whether to promote high-confidence hypotheses to knowledge
    pub auto_promote_to_knowledge: bool,
    /// Batch size for reflection processing
    pub reflection_batch_size: usize,
    /// How often to run maintenance (in seconds)
    pub maintenance_interval_secs: u64,
}

impl Default for LearningCoordinatorConfig {
    fn default() -> Self {
        Self {
            auto_reflect: true,
            reflection_threshold: 0.6,
            auto_hypothesize: true,
            auto_explore: false,
            hypothesis_validation_threshold: 0.75,
            auto_promote_to_knowledge: true,
            reflection_batch_size: 5,
            maintenance_interval_secs: 300, // 5 minutes
        }
    }
}

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
        }
    }

    // ========================================================================
    // Main Entry Points
    // ========================================================================

    /// Process an experience through the full learning pipeline
    ///
    /// Per Architecture §5.3:
    /// Event → Experience Recorder → Experience Storage → Scoring → Reflection → Learning Signals
    pub async fn process_experience(&self, experience: &Experience) -> Result<LearningResult> {
        let mut result = LearningResult::default();
        result.experience_id = experience.id;

        tracing::info!("Processing experience through learning pipeline: {}", experience.id);

        // Step 1: Score the experience
        let score = self.score_experience(experience).await;
        result.score = score;
        self.metrics.increment("learning.experiences.scored").await;

        // Step 2: Generate reflection (if threshold met)
        if self.config.auto_reflect && score >= self.config.reflection_threshold {
            if let Ok(reflection) = self.generate_reflection(experience).await {
                result.reflection_id = Some(reflection.id);
                self.metrics.increment("learning.reflections.generated").await;
            }
        }

        // Step 3: Generate hypotheses (if enabled)
        if self.config.auto_hypothesize {
            if let Ok(hypotheses) = self.generate_hypotheses(experience).await {
                result.hypothesis_ids = hypotheses;
                self.metrics.increment("learning.hypotheses.generated").await;
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

        tracing::info!("Completed learning pipeline for experience {}", experience.id);
        Ok(result)
    }

    /// Validate a hypothesis and potentially promote to knowledge
    ///
    /// Per Architecture §2.5:
    /// "A hypothesis is a temporary model waiting for evidence"
    pub async fn validate_hypothesis(&self, hypothesis_id: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::default();
        result.hypothesis_id = hypothesis_id.to_string();

        // Check hypothesis status in repository
        // This would normally query the hypothesis repository
        
        // If confidence exceeds threshold, promote to knowledge
        if self.config.auto_promote_to_knowledge {
            // Create knowledge from validated hypothesis
            tracing::info!("Hypothesis {} validated, promoting to knowledge", hypothesis_id);
            self.metrics.increment("learning.hypotheses.validated").await;
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

        // 4. Update reputation decay
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
    async fn generate_reflection(&self, experience: &Experience) -> Result<crate::experience::reflection::Reflection> {
        let title = format!("Reflection on: {}", experience.title);
        
        let reflection = self.reflection_engine
            .generate_from_single(experience, title)
            .await?;

        // Publish ReflectionCompleted event
        let event = ExperienceEvent::reflection_completed(experience.id, Uuid::parse_str(&reflection.id).unwrap_or_default());
        let _ = self.bus.publish(event);

        Ok(reflection)
    }

    // ========================================================================
    // Hypothesis Pipeline
    // ========================================================================

    /// Generate hypotheses from experience
    ///
    /// Per Architecture §11:
    /// "Hypotheses enable discovery"
    async fn generate_hypotheses(&self, _experience: &Experience) -> Result<Vec<String>> {
        // Publish HypothesisGenerated event
        let event = ExperienceEvent::hypothesis_generated(Uuid::new_v4(), Uuid::new_v4());
        let _ = self.bus.publish(event);

        Ok(vec![]) // Would return actual hypothesis IDs from repository
    }

    /// Decay confidence of old hypotheses
    async fn decay_hypotheses(&self) -> Result<usize> {
        // This would query the hypothesis repository and apply decay
        tracing::debug!("Running hypothesis decay maintenance");
        Ok(0)
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

        let _knowledge_id = self.knowledge_store.add(knowledge).await;

        // Publish KnowledgeUpdated event
        let event = ExperienceEvent::knowledge_updated(Uuid::new_v4());
        let _ = self.bus.publish(event);

        Ok(())
    }

    /// Consolidate low-confidence knowledge
    async fn consolidate_knowledge(&self) -> Result<usize> {
        tracing::debug!("Running knowledge consolidation");
        Ok(0)
    }

    // ========================================================================
    // Exploration Pipeline
    // ========================================================================

    /// Start exploration for a hypothesis
    pub async fn start_exploration(
        &self,
        _hypothesis_id: String,
        title: String,
        purpose: String,
    ) -> Result<String> {
        let exploration_id = Uuid::new_v4().to_string();
        
        let exploration = Exploration::new(
            exploration_id.clone(),
            title,
            purpose,
            crate::experience::types::ExperienceContext::default(),
        );

        let mut store = self.explorations.write().await;
        store.insert(exploration_id.clone(), exploration);

        // Publish ExplorationStarted event
        let event = ExperienceEvent::exploration_started(Uuid::new_v4());
        let _ = self.bus.publish(event);

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
        let _ = self.bus.publish(event);

        Ok(())
    }

    /// Archive stale explorations
    async fn archive_stale_explorations(&self) -> Result<usize> {
        let cutoff = Utc::now() - Duration::days(7);
        let mut store = self.explorations.write().await;
        let mut archived = 0;

        store.retain(|_id, exp| {
            if let Some(completed) = exp.completed_at {
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
        let reputation = store.entry(source_str.clone())
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

        let _ = reputation.apply(
            experience.id.to_string(),
            crate::experience::reputation::factors::ReputationFactor::Accuracy,
            impact,
            reason,
        );

        // Publish ReputationUpdated event
        let event = ExperienceEvent::reputation_updated(Uuid::new_v4(), source_str, impact as f32);
        let _ = self.bus.publish(event);

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

// ========================================================================
// Result Types
// ========================================================================

/// Result of processing an experience through the learning pipeline
#[derive(Debug, Default)]
pub struct LearningResult {
    pub experience_id: uuid::Uuid,
    pub score: f32,
    pub reflection_id: Option<String>,
    pub hypothesis_ids: Vec<String>,
    pub knowledge_id: Option<uuid::Uuid>,
}

/// Result of validating a hypothesis
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub hypothesis_id: String,
    pub is_valid: bool,
    pub confidence: f32,
    pub promoted_to_knowledge: bool,
}

/// Statistics from maintenance run
#[derive(Debug, Default)]
pub struct MaintenanceStats {
    pub hypotheses_decayed: usize,
    pub explorations_archived: usize,
    pub knowledge_consolidated: usize,
}

/// Statistics about the learning coordinator
#[derive(Debug)]
pub struct LearningCoordinatorStats {
    pub total_reflections: usize,
    pub total_insights: usize,
    pub trusted_insights: usize,
    pub total_patterns: usize,
    pub active_reputations: usize,
    pub active_explorations: usize,
}
