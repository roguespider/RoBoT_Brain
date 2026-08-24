// src/bridge/app/initialization/learning.rs
//! Knowledge store, skills registry, learning coordinator, event subscriber,
//! and reflection pipeline

use std::sync::Arc;

use crate::experience::bus::ExperienceBus;
use crate::experience::encounter_recorder::ExperienceRecorder;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::event_subscriber::EventSubscriber;
use crate::experience::integration::learning_coordinator::LearningCoordinator;
use crate::experience::integration::learning_coordinator::config::LearningCoordinatorConfig;
use crate::experience::metrics::Metrics;
use crate::experience::reflection::ReflectionEngine;
use crate::knowledge::KnowledgeStore;
use crate::skills::registry::SkillRegistry;

/// Result of building the learning pipeline subsystem.
pub(crate) struct LearningPipelineResult {
    pub(crate) knowledge_store: Arc<KnowledgeStore>,
    pub(crate) skills_registry: Arc<SkillRegistry>,
    pub(crate) learning_coordinator: Arc<LearningCoordinator>,
    pub(crate) event_subscriber: Arc<EventSubscriber>,
    pub(crate) reflection_pipeline:
        Arc<crate::experience::integration::reflection_pipeline::ReflectionPipeline>,
}

/// Build the learning pipeline: knowledge store, skills, coordinator, subscriber, reflection.
pub(crate) async fn build_learning_pipeline(
    database: &Arc<crate::database::sqlite::SqliteDatabase>,
    reflection_engine: &Arc<ReflectionEngine>,
    hypothesis_engine_for_subscriber: &Arc<HypothesisEngine>,
    evolution_engine: &Arc<EvolutionEngine>,
    bus: &Arc<ExperienceBus>,
    metrics: &Arc<Metrics>,
    experience_recorder: &Arc<ExperienceRecorder>,
) -> LearningPipelineResult {
    // Create knowledge store
    let knowledge_store = Arc::new(KnowledgeStore::new(10000));

    // Create skills registry
    let skills_registry = Arc::new(SkillRegistry::new());
    skills_registry.load_defaults().await;
    tracing::info!("Skills registry initialized with default skills");

    // Create the Learning Coordinator with an explicit configuration so the
    // tuning knobs (auto_explore, reflection_batch_size,
    // maintenance_interval_secs) are read from a real construction path
    // rather than only from Default.
    let coordinator_config = LearningCoordinatorConfig {
        auto_explore: true,
        reflection_batch_size: 5,
        maintenance_interval_secs: 300,
        ..LearningCoordinatorConfig::default()
    };
    tracing::info!(
        "Learning coordinator config (auto_explore={}, batch_size={}, maintenance_interval={}s)",
        coordinator_config.auto_explore,
        coordinator_config.reflection_batch_size,
        coordinator_config.maintenance_interval_secs
    );
    let learning_coordinator = Arc::new(
        LearningCoordinator::with_config(
            coordinator_config,
            reflection_engine.clone(),
            hypothesis_engine_for_subscriber.clone(),
            knowledge_store.clone(),
            bus.clone(),
            metrics.collector(),
        )
        .with_database(database.clone())
        .with_skill_registry(skills_registry.clone()),
    );
    tracing::info!("Learning coordinator initialized");

    // Create event subscriber for the learning pipeline via the preferred
    // with_learning_coordinator constructor (full §4.04 pipeline driver).
    let event_subscriber_inner = EventSubscriber::with_learning_coordinator(
        learning_coordinator.clone(),
        metrics.collector(),
        reflection_engine.clone(),
        hypothesis_engine_for_subscriber.clone(),
        evolution_engine.clone(),
        knowledge_store.clone(),
        Some(experience_recorder.clone()),
    );
    let event_subscriber = Arc::new(event_subscriber_inner);

    // Create reflection pipeline
    let reflection_pipeline = Arc::new(
        crate::experience::integration::reflection_pipeline::ReflectionPipeline::new(
            reflection_engine.clone(),
            bus.clone(),
        ),
    );
    LearningPipelineResult {
        knowledge_store,
        skills_registry,
        learning_coordinator,
        event_subscriber,
        reflection_pipeline,
    }
}
