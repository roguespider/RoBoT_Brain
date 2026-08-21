// src/experience/integration/event_subscriber/mod.rs

//! Event subscriber that listens to experience events and triggers learning pipeline
//!
//! Per Architecture §4.04:
//! ExperienceRecorded → Reflection observes → Hypothesis evaluates → Knowledge updates → Reputation adjusts
//!
//! This subscriber wires the event bus to all learning subsystems.

mod config;
mod handlers;
mod helpers;
mod reputation;
mod runner;

pub use config::EventSubscriberConfig;
pub use runner::start_event_subscriber;

use std::sync::Arc;

use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::learning_coordinator::LearningCoordinator;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::reputation::reputation::Reputation;
use crate::knowledge::KnowledgeStore;

/// Event subscriber that coordinates the learning pipeline
///
/// This is the main coordinator that wires events to learning subsystems
/// per Architecture §4.04: Experience → Reflection → Hypothesis → Knowledge → Reputation
///
/// Per TASK-V2-01: the subscriber holds an `Arc<LearningCoordinator>` and drives
/// the full Score → Reflect → Hypothesize → Knowledge-promote path on each
/// `ExperienceRecorded` event (the §4.04 single-driver intent), rather than
/// merely re-echoing the event.
pub struct EventSubscriber {
    config: EventSubscriberConfig,
    learning_coordinator: Option<Arc<LearningCoordinator>>,
    metrics: Arc<MetricsCollector>,
    reflection_engine: Arc<ReflectionEngine>,
    hypothesis_engine: Arc<HypothesisEngine>,
    evolution_engine: Arc<EvolutionEngine>,
    knowledge_store: Arc<KnowledgeStore>,
    reputation_store: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Reputation>>>,
    /// Records experiences to the database for structured observation tracking.
    experience_recorder: Option<Arc<ExperienceRecorder>>,
}

impl EventSubscriber {
    /// Create a new event subscriber with dependencies.
    /// Delegates to `with_config_and_coordinator` with default config and no
    /// coordinator so the constructor surface stays consolidated.
    pub fn new(
        metrics: Arc<MetricsCollector>,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
        experience_recorder: Option<Arc<ExperienceRecorder>>,
    ) -> Self {
        Self::with_config_and_coordinator(
            EventSubscriberConfig::default(),
            None,
            metrics,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            experience_recorder,
        )
    }

    /// Create with the learning coordinator that drives the full §4.04 pipeline
    /// (TASK-V2-01). This is the preferred constructor: the subscriber consumes
    /// each `ExperienceRecorded` event once and runs the complete
    /// Score → Reflect → Hypothesize → Knowledge-promote path.
    /// Delegates to `with_config_and_coordinator` with default config.
    pub fn with_learning_coordinator(
        learning_coordinator: Arc<LearningCoordinator>,
        metrics: Arc<MetricsCollector>,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
        experience_recorder: Option<Arc<ExperienceRecorder>>,
    ) -> Self {
        Self::with_config_and_coordinator(
            EventSubscriberConfig::default(),
            Some(learning_coordinator),
            metrics,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            experience_recorder,
        )
    }

    /// Whether a learning coordinator is attached (drives the full §4.04
    /// pipeline when present).
    pub fn has_learning_coordinator(&self) -> bool {
        self.learning_coordinator.is_some()
    }

    /// Create with custom config and the learning coordinator attached.
    /// Combines the tuning surface of `with_config` with the full §4.04
    /// pipeline driver of `with_learning_coordinator`.
    pub fn with_config_and_coordinator(
        config: EventSubscriberConfig,
        learning_coordinator: Option<Arc<LearningCoordinator>>,
        metrics: Arc<MetricsCollector>,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
        experience_recorder: Option<Arc<ExperienceRecorder>>,
    ) -> Self {
        Self {
            config,
            learning_coordinator,
            metrics,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            reputation_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            experience_recorder,
        }
    }
}
