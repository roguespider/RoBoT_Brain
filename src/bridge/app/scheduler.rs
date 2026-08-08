// src/bridge/app/scheduler.rs
//! Scheduler setup and task handler registration

use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;
use crate::experience::evolution::EvolutionEngine;
use crate::experience::hypothesis::HypothesisEngine;
use crate::experience::metrics::MetricsCollector;
use crate::experience::reflection::ReflectionEngine;
use crate::experience::reputation::decay::ReputationDecay;
use crate::experience::scheduler::{Scheduler, TaskSchedule, TaskType};
use crate::memory::MemoryRetrieval;

/// Setup background task scheduler with default tasks.
pub async fn setup_scheduler(database: Arc<SqliteDatabase>) -> Result<Arc<Scheduler>> {
    let scheduler = Arc::new(Scheduler::new(database));

    // Schedule periodic reflection (every 30 minutes)
    scheduler
        .create_task(
            "periodic_reflection",
            TaskType::Reflection,
            TaskSchedule::Interval { seconds: 1800 },
        )
        .await?;

    // Schedule hypothesis evaluation (every hour)
    scheduler
        .create_task(
            "hypothesis_evaluation",
            TaskType::HypothesisEvaluation,
            TaskSchedule::Interval { seconds: 3600 },
        )
        .await?;

    // Schedule metrics collection (every 5 minutes)
    scheduler
        .create_task(
            "metrics_collection",
            TaskType::MetricsCollection,
            TaskSchedule::Interval { seconds: 300 },
        )
        .await?;

    // Schedule evolution maintenance (every day at midnight)
    scheduler
        .create_task(
            "evolution_maintenance",
            TaskType::EvolutionMaintenance,
            TaskSchedule::Daily { hour: 0, minute: 0 },
        )
        .await?;

    // Schedule memory consolidation (every hour)
    scheduler
        .create_task(
            "memory_consolidation",
            TaskType::MemoryConsolidation,
            TaskSchedule::Interval { seconds: 3600 },
        )
        .await?;

    // Schedule memory checkpoint (every 15 minutes)
    // Per Architecture §6.3: SQLite is the persistence layer
    scheduler
        .create_task(
            "memory_checkpoint",
            TaskType::MemoryCheckpoint,
            TaskSchedule::Interval { seconds: 900 },
        )
        .await?;

    Ok(scheduler)
}

/// Register task handlers for the scheduler
pub async fn register_task_handlers(
    scheduler: Arc<Scheduler>,
    memory_retrieval: Arc<MemoryRetrieval>,
    reflection_engine: Arc<ReflectionEngine>,
    hypothesis_engine: Arc<Mutex<HypothesisEngine>>,
    evolution_engine: Arc<EvolutionEngine>,
    metrics: Arc<MetricsCollector>,
    database: Arc<SqliteDatabase>,
) {
    use crate::experience::scheduler::TaskType;

    // Reflection task handler - analyzes recent experiences for patterns
    let reflection_engine_clone = reflection_engine.clone();
    let database_reflect = database.clone();
    scheduler
        .register_handler(
            TaskType::Reflection,
            Box::new(move || {
                let reflection_engine = reflection_engine_clone.clone();
                let database = database_reflect.clone();
                Box::pin(async move {
                    tracing::info!("Executing scheduled reflection task");

                    // Load recent experiences from database
                    let experiences = crate::database::queries::list_experiences(&database.connection().unwrap(), 100)
                        .unwrap_or_default();

                    if experiences.len() >= 3 {
                        // Analyze experiences for patterns
                        match reflection_engine.analyze_experiences(&experiences).await {
                            Ok(report) => {
                                tracing::info!(
                                    "Reflection analysis complete: {} patterns, {} themes found",
                                    report.patterns.len(),
                                    report.themes.len()
                                );
                            }
                            Err(e) => {
                                tracing::error!("Reflection analysis failed: {}", e);
                            }
                        }
                    }

                    // Archive old reflections
                    let archived = reflection_engine.archive_old(30).await.unwrap_or(0);
                    if archived > 0 {
                        tracing::info!("Archived {} old reflections", archived);
                    }

                    Ok(())
                })
            }),
        )
        .await;

    // Hypothesis evaluation handler - evaluates and updates hypothesis confidence
    let hypothesis_engine_clone = hypothesis_engine.clone();
    scheduler
        .register_handler(
            TaskType::HypothesisEvaluation,
            Box::new(move || {
                let hypothesis_engine = hypothesis_engine_clone.clone();
                Box::pin(async move {
                    tracing::info!("Executing scheduled hypothesis evaluation");

                    // Perform hypothesis maintenance
                    let engine_result = hypothesis_engine.lock();
                    match engine_result {
                        Ok(mut engine) => {
                            if let Err(e) = engine.maintenance() {
                                tracing::error!("Hypothesis evaluation failed: {}", e);
                            }
                        }
                        Err(poisoned) => {
                            tracing::error!("Hypothesis engine mutex poisoned");
                            if let Err(e) = poisoned.into_inner().maintenance() {
                                tracing::error!("Hypothesis evaluation failed on recovered mutex: {}", e);
                            }
                        }
                    }

                    Ok(())
                })
            }),
        )
        .await;

    // Metrics collection handler - aggregates and reports metrics
    let metrics_clone = metrics.clone();
    scheduler
        .register_handler(
            TaskType::MetricsCollection,
            Box::new(move || {
                let metrics = metrics_clone.clone();
                Box::pin(async move {
                    tracing::debug!("Executing scheduled metrics collection");

                    // Get metrics summary
                    let summary = metrics.summary().await;
                    tracing::debug!(
                        "Metrics: {} counters, {} gauges, {} metrics tracked",
                        summary.counters.len(),
                        summary.gauges.len(),
                        summary.metrics.len()
                    );

                    // Clear old metrics (older than 24 hours)
                    metrics.clear_old(24).await;

                    Ok(())
                })
            }),
        )
        .await;

    // Evolution maintenance handler - promotes/demotes behaviors based on performance
    let evolution_engine_clone = evolution_engine.clone();
    scheduler
        .register_handler(
            TaskType::EvolutionMaintenance,
            Box::new(move || {
                let evolution_engine = evolution_engine_clone.clone();
                Box::pin(async move {
                    tracing::info!("Executing scheduled evolution maintenance");

                    // Evaluate and maintain all behaviors for promotion/demotion
                    match evolution_engine.evaluate_and_maintain().await {
                        Ok(summary) => {
                            tracing::info!(
                                "Evolution maintenance: {} total, {} promoted, {} deprecated, {} integrated",
                                summary.total_behaviors,
                                summary.promoted,
                                summary.deprecated,
                                summary.integrated
                            );
                        }
                        Err(e) => {
                            tracing::error!("Evolution maintenance failed: {}", e);
                        }
                    }

                    // Archive deprecated behaviors
                    let archived = evolution_engine.archive_deprecated().await.unwrap_or(0);
                    if archived > 0 {
                        tracing::info!("Archived {} deprecated behaviors", archived);
                    }

                    Ok(())
                })
            }),
        )
        .await;

    // Exploration analysis handler - analyzes exploration results and patterns
    scheduler
        .register_handler(
            TaskType::ExplorationAnalysis,
            Box::new(|| {
                Box::pin(async move {
                    tracing::debug!("Executing scheduled exploration analysis");

                    // Exploration analysis would analyze exploration results
                    // For now, log that the task executed
                    tracing::debug!("Exploration analysis completed");

                    Ok(())
                })
            }),
        )
        .await;

    // Cleanup handler - removes old/stale data
    scheduler
        .register_handler(
            TaskType::Cleanup,
            Box::new(move || {
                Box::pin(async move {
                    tracing::info!("Executing scheduled cleanup");

                    // Clean up old events from database
                    let cutoff = chrono::Utc::now() - chrono::Duration::days(7);

                    // This would call cleanup queries if implemented
                    tracing::info!("Cleanup task executed (older than {})", cutoff);

                    Ok(())
                })
            }),
        )
        .await;

    // Reputation decay handler - applies time-based reputation decay
    let database_reput = database.clone();
    scheduler
        .register_handler(
            TaskType::ReputationDecay,
            Box::new(move || {
                let database = database_reput.clone();
                Box::pin(async move {
                    tracing::debug!("Executing scheduled reputation decay");

                    // Load reputations and apply decay
                    let reputations =
                        crate::database::queries::list_reputations(&database.connection().unwrap())
                            .unwrap_or_default();

                    let mut decayed_count = 0;
                    for mut reputation in reputations {
                        let original_score = reputation.score;
                        reputation.score =
                            ReputationDecay::apply(reputation.score, reputation.updated_at);

                        if (original_score - reputation.score).abs() > 0.001 {
                            decayed_count += 1;
                            // Save updated reputation
                            if let Err(e) = crate::database::queries::insert_reputation(
                                &database.connection().unwrap(),
                                &reputation,
                            ) {
                                tracing::warn!(
                                    "Failed to update reputation {}: {}",
                                    reputation.id,
                                    e
                                );
                            }
                        }
                    }

                    if decayed_count > 0 {
                        tracing::debug!("Applied decay to {} reputations", decayed_count);
                    }

                    Ok(())
                })
            }),
        )
        .await;

    // Memory consolidation handler
    // Per Architecture §6.3: Consolidates Working Memory to Permanent Memory
    let memory_retrieval_clone = memory_retrieval.clone();
    let database_clone = database.clone();
    scheduler
        .register_handler(
            TaskType::MemoryConsolidation,
            Box::new(move || {
                let memory_retrieval = memory_retrieval_clone.clone();
                let database = database_clone.clone();
                Box::pin(async move {
                    tracing::info!("Executing scheduled memory consolidation");

                    // Consolidate between in-memory caches (Architecture §6.3)
                    let stats = memory_retrieval.consolidate().await;
                    tracing::info!(
                        "Memory consolidation complete: {} promoted, {} archived, {} deleted, {} kept",
                        stats.promoted, stats.archived, stats.deleted, stats.kept
                    );

                    // Checkpoint caches to database for persistence
                    if let Err(e) = memory_retrieval.checkpoint_to_database(&database).await {
                        tracing::error!("Failed to checkpoint memories to database: {}", e);
                    }

                    Ok(())
                })
            }),
        )
        .await;

    // Memory checkpoint handler
    // Per Architecture §6.3: Checkpoint in-memory caches to SQLite
    let memory_retrieval_checkpoint = memory_retrieval.clone();
    let database_checkpoint = database.clone();
    scheduler
        .register_handler(
            TaskType::MemoryCheckpoint,
            Box::new(move || {
                let memory_retrieval = memory_retrieval_checkpoint.clone();
                let database = database_checkpoint.clone();
                Box::pin(async move {
                    tracing::debug!("Executing scheduled memory checkpoint");

                    // Checkpoint caches to database for persistence
                    if let Err(e) = memory_retrieval.checkpoint_to_database(&database).await {
                        tracing::error!("Failed to checkpoint memories to database: {}", e);
                        return Err(e);
                    }

                    tracing::debug!("Memory checkpoint completed successfully");
                    Ok(())
                })
            }),
        )
        .await;

    tracing::info!("Registered {} task handlers", 9);
}
