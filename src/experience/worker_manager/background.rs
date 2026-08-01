// /src/experience/worker_manager/background.rs
//! Background task for the worker manager

use std::sync::Arc;
use tokio::sync::broadcast;

use crate::experience::bus::ExperienceBus;
use super::manager::WorkerManager;

/// Start the worker manager as a background task
/// Subscribes to the event bus and enqueues jobs for all observers
pub fn start_worker_manager(
    bus: Arc<ExperienceBus>,
    manager: Arc<WorkerManager>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut receiver = bus.subscribe();
        tracing::info!("Worker manager started, listening for events");

        loop {
            match receiver.recv().await {
                Ok(event) => {
                    // Broadcast to all workers - they will filter based on accepts()
                    if let Err(e) = manager.broadcast_event(event).await {
                        tracing::error!("Failed to broadcast event: {}", e);
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("Worker manager lagged {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    tracing::info!("Event bus closed, worker manager shutting down");
                    break;
                }
            }
        }
    })
}
