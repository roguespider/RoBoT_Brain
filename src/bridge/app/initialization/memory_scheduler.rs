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

/// Build memory system and scheduler.
pub(crate) async fn build_memory_scheduler(
    database: &Arc<SqliteDatabase>,
    knowledge_store: &Arc<KnowledgeStore>,
    learning_coordinator: &Arc<LearningCoordinator>,
    reflection_engine: &Arc<ReflectionEngine>,
    hypothesis_engine: &Arc<Mutex<HypothesisEngine>>,
    evolution_engine: &Arc<EvolutionEngine>,
    metrics: &Arc<Metrics>,
) -> anyhow::Result<MemorySchedulerResult> {
    // Create memory system
    let working_memory_core = Arc::new(MemWorkingMemory::new(1000));
    let permanent_memory = Arc::new(PermanentMemory::new(10000));
    let memory_retrieval = Arc::new(MemoryRetrieval::new(
        working_memory_core.clone(),
        permanent_memory.clone(),
    ));

    // Create memory pipeline
    let memory_pipeline = Arc::new(crate::memory::pipeline::MemoryPipeline::new(
        database.clone(),
    ));

    // Load memories from database
    if let Err(e) = working_memory_core.load_from_database(database).await {
        tracing::warn!("Failed to load working memory from database: {}", e);
    }
    if let Err(e) = permanent_memory.load_from_database(database).await {
        tracing::warn!("Failed to load permanent memory from database: {}", e);
    }
    // Log knowledge-store health at startup so the shared KnowledgeStore handle
    // (already wired into the LearningCoordinator by the caller) is verified live.
    let kstats = knowledge_store.stats().await;
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
    let scheduler = crate::bridge::app::scheduler::setup_scheduler(database.clone()).await?;

    // Register task handlers
    crate::bridge::app::scheduler::register_task_handlers(
        crate::bridge::app::scheduler::SchedulerTaskSystems {
            scheduler: scheduler.clone(),
            memory_retrieval: memory_retrieval.clone(),
            reflection_engine: reflection_engine.clone(),
            hypothesis_engine: hypothesis_engine.clone(),
            evolution_engine: evolution_engine.clone(),
            metrics: metrics.collector(),
            database: database.clone(),
            learning_coordinator: learning_coordinator.clone(),
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
