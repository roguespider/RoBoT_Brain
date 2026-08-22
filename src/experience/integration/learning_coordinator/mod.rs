// src/experience/integration/learning_coordinator/mod.rs
//! Learning Coordinator - Orchestrates the complete learning pipeline
//!
//! Per Architecture §9 - Learning Engine:
//! Input → Observation → Memory → Experience → Knowledge → Planning → Decision → Action → Reflection
//!
//! This coordinator ensures the continuous feedback loop:
//! Experience → Reflection → Hypothesis → Validation → Knowledge Update → Behavior Improvement

pub mod config;
pub mod entry;
pub mod exploration;
pub mod generalization;
pub mod hypothesis;
pub mod knowledge;
pub mod reinforcement;
pub mod reputation;
pub mod results;

pub use config::LearningCoordinatorConfig;
pub use entry::EntryMethods;
pub use exploration::ExplorationManager;
pub use reputation::ReputationManager;
pub use results::{LearningCoordinatorStats, LearningResult, MaintenanceStats, ValidationResult};

use anyhow::Result;
use std::sync::Arc;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::bus::ExperienceBus;
use crate::experience::exploration::Exploration;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::reputation::score::Reputation;
use crate::experience::types::Experience;
use crate::knowledge::KnowledgeStore;
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
    ///
    /// Delegates to [`with_config`] with the default configuration; kept as the
    /// canonical default-construction entry point (Architecture §9).
    pub fn new(
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        knowledge_store: Arc<KnowledgeStore>,
        bus: Arc<ExperienceBus>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self::with_config(
            LearningCoordinatorConfig::default(),
            reflection_engine,
            hypothesis_engine,
            knowledge_store,
            bus,
            metrics,
        )
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
    // Main Entry Points (delegated to EntryMethods)
    // ========================================================================

    /// Process an experience through the full learning pipeline
    ///
    /// Per Architecture §5.3:
    /// Event → Experience Recorder → Experience Storage → Scoring → Reflection → Learning Signals
    pub async fn process_experience(&self, experience: &Experience) -> Result<LearningResult> {
        let methods = EntryMethods {
            config: &self.config,
            hypothesis_engine: &self.hypothesis_engine,
            reflection_engine: &self.reflection_engine,
            knowledge_store: &self.knowledge_store,
            reputations: &self.reputations,
            explorations: &self.explorations,
            metrics: &self.metrics,
            bus: &self.bus,
            skill_registry: &self.skill_registry,
        };
        methods.process_experience(experience).await
    }

    /// Process an experience through the full learning pipeline (extended version)
    pub async fn process_experience_full(&self, experience: &Experience) -> Result<LearningResult> {
        let methods = EntryMethods {
            config: &self.config,
            hypothesis_engine: &self.hypothesis_engine,
            reflection_engine: &self.reflection_engine,
            knowledge_store: &self.knowledge_store,
            reputations: &self.reputations,
            explorations: &self.explorations,
            metrics: &self.metrics,
            bus: &self.bus,
            skill_registry: &self.skill_registry,
        };
        methods.process_experience_full(experience).await
    }

    /// Validate a hypothesis and potentially promote to knowledge
    pub async fn validate_hypothesis(&self, hypothesis_id: &str) -> Result<ValidationResult> {
        let methods = EntryMethods {
            config: &self.config,
            hypothesis_engine: &self.hypothesis_engine,
            reflection_engine: &self.reflection_engine,
            knowledge_store: &self.knowledge_store,
            reputations: &self.reputations,
            explorations: &self.explorations,
            metrics: &self.metrics,
            bus: &self.bus,
            skill_registry: &self.skill_registry,
        };
        methods.validate_hypothesis(hypothesis_id).await
    }

    /// Perform maintenance tasks (called periodically)
    pub async fn run_maintenance(&self) -> Result<MaintenanceStats> {
        // Record active-pipeline counts as maintenance diagnostics (Architecture
        // §2.7/§12 observability) so the exploration/reputation introspection
        // accessors stay wired to a real caller rather than dead state.
        let explorations = self.active_exploration_count().await;
        let reputations = self.active_reputation_count().await;
        tracing::debug!(
            "Maintenance pre-check: {} active explorations, {} reputation sources",
            explorations,
            reputations
        );
        let methods = EntryMethods {
            config: &self.config,
            hypothesis_engine: &self.hypothesis_engine,
            reflection_engine: &self.reflection_engine,
            knowledge_store: &self.knowledge_store,
            reputations: &self.reputations,
            explorations: &self.explorations,
            metrics: &self.metrics,
            bus: &self.bus,
            skill_registry: &self.skill_registry,
        };
        methods.run_maintenance().await
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
        let manager = ExplorationManager::new(self.explorations.clone(), self.bus.clone());
        manager
            .start_exploration(hypothesis_id, title, purpose)
            .await
    }

    /// Complete an exploration
    pub async fn complete_exploration(&self, exploration_id: &str) -> Result<()> {
        let manager = ExplorationManager::new(self.explorations.clone(), self.bus.clone());
        manager.complete_exploration(exploration_id).await
    }

    /// Number of currently active explorations (Architecture §2.7 diagnostics).
    pub async fn active_exploration_count(&self) -> usize {
        let manager = ExplorationManager::new(self.explorations.clone(), self.bus.clone());
        manager.active_count().await
    }

    // ========================================================================
    // Reputation Pipeline
    // ========================================================================

    /// Update reputation based on experience outcome
    ///
    /// Per Architecture §12:
    /// "Reputation determines how much each source of knowledge should be trusted"
    pub async fn update_reputation(&self, experience: &Experience) -> Result<()> {
        let manager = ReputationManager::new(self.reputations.clone(), self.bus.clone());
        manager.update_reputation(experience).await
    }

    /// Get reputation for a source
    pub async fn get_reputation(&self, source: &str) -> Option<f64> {
        let manager = ReputationManager::new(self.reputations.clone(), self.bus.clone());
        manager.get_reputation(source).await
    }

    /// Number of sources with active reputation records (Architecture §12).
    pub async fn active_reputation_count(&self) -> usize {
        let manager = ReputationManager::new(self.reputations.clone(), self.bus.clone());
        manager.active_count().await
    }

    // ========================================================================
    // Generalization
    // ========================================================================

    /// Extract common patterns from a set of experiences (Architecture §9:
    /// "Generalization extracts common patterns from specific instances").
    pub fn extract_patterns(
        &self,
        experiences: &[Experience],
    ) -> Vec<crate::experience::integration::learning_coordinator::results::LearningPattern> {
        use crate::experience::integration::learning_coordinator::generalization::GeneralizationMethods;
        let methods = GeneralizationMethods {
            knowledge_store: &self.knowledge_store,
            bus: &self.bus,
            metrics: &self.metrics,
        };
        methods.extract_common_patterns(experiences)
    }

    // ========================================================================
    // Stats
    // ========================================================================

    /// Get coordinator statistics
    pub async fn get_stats(&self) -> LearningCoordinatorStats {
        let reflections = self.reflection_engine.get_stats().await;
        let reputations = self.reputations.read().await;
        let explorations = self.explorations.read().await;

        // Surface mature-pattern count in the trace so EngineStats::
        // mature_patterns is consumed by a real caller (Architecture §10
        // observability: pattern maturity gates insight promotion).
        tracing::debug!(
            "Reflection engine stats: {} patterns ({} mature)",
            reflections.total_patterns,
            reflections.mature_patterns
        );

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
