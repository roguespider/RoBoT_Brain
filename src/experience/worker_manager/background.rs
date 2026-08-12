// /src/experience/worker_manager/background.rs
//! Background task for the worker manager

use std::sync::Arc;
use tokio::sync::broadcast;

use super::manager::WorkerManager;
use crate::experience::bus::ExperienceBus;

/// Start the worker manager as a background task
/// Subscribes to the event bus and enqueues jobs for all observers
pub fn start_worker_manager(
    bus: Arc<ExperienceBus>,
    manager: Arc<WorkerManager>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut receiver = bus.subscribe();
        tracing::info!("Worker manager started, listening for events");
        tracing::debug!("Event bus subscriber count: {}", bus.subscriber_count());

        loop {
            match receiver.recv().await {
                Ok(event) => {
                    // Broadcast to all workers - they will filter based on accepts()
                    let event_id = event.id;
                    match manager.broadcast_event(event).await {
                        Ok(()) => {
                            // Mark the job as completed in the durable queue
                            if let Err(e) = manager.mark_job_complete(&event_id.to_string()) {
                                tracing::debug!("Failed to mark job complete: {}", e);
                            }
                        }
                        Err(e) => {
                            // Mark the job as failed if broadcast failed
                            if let Err(mk_err) =
                                manager.mark_job_failed(&event_id.to_string(), e.to_string())
                            {
                                tracing::debug!("Failed to mark job failed: {}", mk_err);
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Worker manager lagged {} events", n);
                    // Drain the lagged events so we don't re-process the same one,
                    // then record a failed job for the skipped events.
                    for _ in 0..n {
                        let _ = receiver.recv().await;
                    }
                    if let Err(e) = manager.mark_job_failed(
                        &format!("lagged_{}", n),
                        format!("Worker manager lagged {} events", n),
                    ) {
                        tracing::debug!("Failed to mark lagged job: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Event bus closed, worker manager shutting down");
                    break;
                }
            }
        }
        bus.unsubscribe();
    })
}
