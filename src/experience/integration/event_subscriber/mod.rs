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

use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::reputation::reputation::Reputation;
use crate::experience::integration::learning_coordinator::LearningCoordinator;
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
}

impl EventSubscriber {
    /// Create a new event subscriber with dependencies
    pub fn new(
        metrics: Arc<MetricsCollector>,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
    ) -> Self {
        Self {
            config: EventSubscriberConfig::default(),
            learning_coordinator: None,
            metrics,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            reputation_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with the learning coordinator that drives the full §4.04 pipeline
    /// (TASK-V2-01). This is the preferred constructor: the subscriber consumes
    /// each `ExperienceRecorded` event once and runs the complete
    /// Score → Reflect → Hypothesize → Knowledge-promote path.
    pub fn with_learning_coordinator(
        learning_coordinator: Arc<LearningCoordinator>,
        metrics: Arc<MetricsCollector>,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
    ) -> Self {
        Self {
            config: EventSubscriberConfig::default(),
            learning_coordinator: Some(learning_coordinator),
            metrics,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            reputation_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with custom config
    pub fn with_config(
        config: EventSubscriberConfig,
        metrics: Arc<MetricsCollector>,
        reflection_engine: Arc<ReflectionEngine>,
        hypothesis_engine: Arc<HypothesisEngine>,
        evolution_engine: Arc<EvolutionEngine>,
        knowledge_store: Arc<KnowledgeStore>,
    ) -> Self {
        Self {
            config,
            learning_coordinator: None,
            metrics,
            reflection_engine,
            hypothesis_engine,
            evolution_engine,
            knowledge_store,
            reputation_store: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}
