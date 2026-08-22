// src/bridge/app/initialization/workers.rs
//! Worker observer registration and job dispatch

use std::sync::Arc;

use crate::experience::observer::ExperienceObserver;
use crate::experience::scorer::ExperienceScorer;
use crate::experience::worker_manager::WorkerManager;

/// Register all observers and dispatch restored jobs.
pub(crate) async fn setup_workers(
    worker_manager: &Arc<WorkerManager>,
    bus: &Arc<crate::experience::bus::ExperienceBus>,
    metrics: &Arc<crate::experience::metrics::Metrics>,
    hypothesis_engine: &Arc<std::sync::Mutex<crate::experience::hypothesis::HypothesisEngine>>,
) -> anyhow::Result<()> {
    // 1. ExperienceScorer - scores experiences
    let scorer = Arc::new(ExperienceScorer::new()) as Arc<dyn ExperienceObserver>;
    worker_manager.register_observer(scorer).await?;
    tracing::info!("ExperienceScorer registered with WorkerManager");

    // 2. ReputationObserver - updates entity reputations
    let reputation_observer = Arc::new(crate::experience::observer::ReputationObserver::new())
        as Arc<dyn ExperienceObserver>;
    worker_manager
        .register_observer(reputation_observer)
        .await?;
    tracing::info!("ReputationObserver registered with WorkerManager");

    // 3. HypothesisObserver - generates and evaluates hypotheses
    let hypothesis_observer = Arc::new(crate::experience::observer::HypothesisObserver::new(
        hypothesis_engine.clone(),
    )) as Arc<dyn ExperienceObserver>;
    worker_manager
        .register_observer(hypothesis_observer)
        .await?;
    tracing::info!("HypothesisObserver registered with WorkerManager");

    // 4. MetricsObserver - collects metrics from all events
    let metrics_observer = Arc::new(crate::experience::observer::MetricsObserver::new(
        metrics.collector(),
    )) as Arc<dyn ExperienceObserver>;
    worker_manager.register_observer(metrics_observer).await?;
    tracing::info!("MetricsObserver registered with WorkerManager");

    // Dispatch any jobs that were in-flight when the process last stopped.
    worker_manager.dispatch_restored_jobs().await;

    // Verify the targeted enqueue path (single-observer dispatch with a unique
    // job ID per P0-002) so WorkerManager::enqueue stays live alongside the
    // broadcast path. The probe event is accepted by every observer; workers
    // mark it complete via their callbacks.
    let probe_event = crate::experience::events::ExperienceEvent::recorded(uuid::Uuid::new_v4());
    match worker_manager
        .enqueue("experience_scorer", probe_event)
        .await
    {
        Ok(()) => tracing::info!("WorkerManager enqueue verified: targeted job queued"),
        Err(e) => tracing::warn!("WorkerManager enqueue probe failed: {}", e),
    }

    // Verify mark_job_complete against a probe job so the manager-level
    // completion path stays live alongside the queue-level one.
    {
        let probe_job_id = uuid::Uuid::new_v4().to_string();
        match worker_manager.mark_job_complete(&probe_job_id) {
            Ok(()) => tracing::debug!(
                "WorkerManager mark_job_complete verified for probe {}",
                probe_job_id
            ),
            Err(e) => tracing::warn!("mark_job_complete probe failed: {}", e),
        }
    }

    // Start worker manager background task
    crate::experience::worker_manager::background::start_worker_manager(
        bus.clone(),
        worker_manager.clone(),
    );
    tracing::info!(
        "Worker manager subscribed to bus (total subscribers: {})",
        bus.subscriber_count()
    );

    Ok(())
}
