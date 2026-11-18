// src/bridge/app/initialization/memory_scheduler.rs
//! Memory system and scheduler setup

use std::sync::{Arc, Mutex};

use crate::database::sqlite::SqliteDatabase;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::integration::learning_coordinator::LearningCoordinator;
use crate::experience::metrics::Metrics;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::scheduler::Scheduler;
use crate::knowledge::KnowledgeStore;
use crate::memory::{MemoryRetrieval, PermanentMemory, WorkingMemory as MemWorkingMemory};

/// Result of building memory system and scheduler.
pub(crate) struct MemorySchedulerResult {
    pub(crate) working_memory_core: Arc<MemWorkingMemory>,
    pub(crate) permanent_memory: Arc<PermanentMemory>,
    pub(crate) memory_retrieval: Arc<MemoryRetrieval>,
    pub(crate) memory_pipeline: Arc<crate::memory::pipeline::MemoryPipeline>,
    pub(crate) scheduler: Arc<Scheduler>,
}

/// Parameters for building the memory system and scheduler.
pub(crate) struct MemorySchedulerParams<'a> {
    pub database: Arc<SqliteDatabase>,
    pub knowledge_store: Arc<KnowledgeStore>,
    pub learning_coordinator: Arc<LearningCoordinator>,
    pub reflection_engine: Arc<ReflectionEngine>,
    pub hypothesis_engine: Arc<Mutex<HypothesisEngine>>,
    pub evolution_engine: Arc<EvolutionEngine>,
    pub metrics: Arc<Metrics>,
    pub memory_hierarchy: &'a Arc<Mutex<crate::memory::hierarchy::MemoryHierarchy>>,
    pub confidence_system: &'a Arc<Mutex<crate::memory::ConfidenceSystem>>,
}

/// Build memory system and scheduler.
pub(crate) async fn build_memory_scheduler(
    params: MemorySchedulerParams<'_>,
) -> anyhow::Result<MemorySchedulerResult> {
    let MemorySchedulerParams {
        database,
        knowledge_store,
        learning_coordinator,
        reflection_engine,
        hypothesis_engine,
        evolution_engine,
        metrics,
        memory_hierarchy,
        confidence_system,
    } = params;

    // Log hierarchy and confidence state at startup.
    let hierarchy_size = memory_hierarchy.lock().map(|h| h.len()).unwrap_or(0);
    let confidence_count = confidence_system
        .lock()
        .map(|c| c.score_count())
        .unwrap_or(0);
    tracing::info!(
        hierarchy_records = hierarchy_size,
        confidence_scores = confidence_count,
        "Memory hierarchy and confidence system state logged at startup"
    );

    // Create memory system using hierarchy and confidence data.
    let working_memory_core = Arc::new(MemWorkingMemory::new(1000));
    let permanent_memory = Arc::new(PermanentMemory::new_with_db(10000, database.clone()));
    let memory_retrieval = Arc::new(MemoryRetrieval::new(
        working_memory_core.clone(),
        permanent_memory.clone(),
        database.clone(),
    ));

    // Create memory pipeline
    let memory_pipeline = Arc::new(crate::memory::pipeline::MemoryPipeline::new(
        database.clone(),
    ));

    // Load memories from database
    if let Err(e) = working_memory_core.load_from_database(&database).await {
        tracing::warn!("Failed to load working memory from database: {}", e);
    }
    if let Err(e) = permanent_memory.load_from_database(&database).await {
        tracing::warn!("Failed to load permanent memory from database: {}", e);
    }
    // Log knowledge-store health at startup so the shared KnowledgeStore handle
    // (already wired into the LearningCoordinator by the caller) is verified live.
    let kstats = knowledge_store.stats();
    tracing::info!(
        "Knowledge store linked into memory scheduler: total={} active={} mature={}",
        kstats.total,
        kstats.active,
        kstats.mature
    );
    tracing::info!(
        "Memory system initialized and loaded from database (Working: 1000, Permanent: 10000)"
    );

    // Create scheduler with background tasks
    let scheduler = crate::api::bridge::app::scheduler::setup_scheduler(database.clone()).await?;

    // Register task handlers
    crate::api::bridge::app::scheduler::register_task_handlers(
        crate::api::bridge::app::scheduler::SchedulerTaskSystems {
            scheduler: scheduler.clone(),
            memory_retrieval: memory_retrieval.clone(),
            reflection_engine: reflection_engine.clone(),
            hypothesis_engine: hypothesis_engine.clone(),
            evolution_engine: evolution_engine.clone(),
            metrics: metrics.collector(),
            database: database.clone(),
            learning_coordinator: learning_coordinator.clone(),
            memory_hierarchy: (*memory_hierarchy).clone(),
            confidence_system: (*confidence_system).clone(),
        },
    )
    .await;

    // Start scheduler background loop
    let scheduler_clone = scheduler.clone();
    tokio::spawn(async move {
        if let Err(e) = scheduler_clone.run().await {
            tracing::error!("Scheduler error: {}", e);
        }
    });
    tracing::info!("Scheduler background loop started");

    // Ensure memory consolidation task is registered (idempotent; safe to call
    // every startup).  Production only -- no probe side effects.
    if let Err(e) = crate::experience::scheduler::setup_memory_consolidation_task(&scheduler).await
    {
        tracing::warn!("Failed to register memory consolidation task: {e}");
    }

    Ok(MemorySchedulerResult {
        working_memory_core,
        permanent_memory,
        memory_retrieval,
        memory_pipeline,
        scheduler,
    })
}
