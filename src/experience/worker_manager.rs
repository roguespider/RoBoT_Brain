// /src/experience/worker_manager.rs
//! Worker Manager - manages workers per observer per Architecture §22
//!
//! Design per README.md Pipeline Design:
//! Experience Recorded → Recorder → Bus → Job Queue → Workers → Observers
//!
//! This module connects the event bus to workers, routing events to appropriate
//! observers based on their acceptance criteria.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::observer::ExperienceObserver;
use crate::experience::worker::{ExperienceWorker, ObserverJob, WorkerStats};

/// Maximum jobs that can be queued per observer
const MAX_QUEUE_DEPTH: usize = 100;

/// Manages workers for all registered observers
/// Routes events from bus to appropriate worker channels
pub struct WorkerManager {
    /// Event bus for subscribing to events
    bus: Arc<ExperienceBus>,
    /// Workers indexed by observer name
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
}

/// Handle to a worker's sender and stats
struct WorkerHandle {
    /// Sender to enqueue jobs for this worker
    sender: mpsc::Sender<ObserverJob>,
    /// Statistics for this worker
    stats: Arc<WorkerStats>,
    /// Tokio handle for the worker task
    _task: tokio::task::JoinHandle<()>,
}

impl WorkerManager {
    /// Create a new worker manager
    pub fn new(bus: Arc<ExperienceBus>) -> Self {
        Self {
            bus,
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an observer and spawn a worker for it
    pub async fn register_observer(
        &self,
        observer: Arc<dyn ExperienceObserver>,
    ) -> Result<()> {
        let name = observer.name().to_string();
        
        let mut workers = self.workers.write().await;
        
        // Check if worker already exists for this observer
        if workers.contains_key(&name) {
            tracing::warn!("Observer {} already registered, skipping", name);
            return Ok(());
        }

        // Create channel for this worker
        let (sender, receiver) = mpsc::channel(MAX_QUEUE_DEPTH);

        // Create the worker
        let worker = ExperienceWorker::new(observer, receiver);
        let stats = worker.stats();

        // Clone name for the async task
        let task_name = name.clone();
        
        // Spawn the worker task
        let task = tokio::spawn(async move {
            if let Err(e) = worker.start().await {
                tracing::error!("Worker for {} stopped with error: {}", task_name, e);
            }
        });

        // Store the handle
        workers.insert(name.clone(), WorkerHandle {
            sender,
            stats,
            _task: task,
        });

        tracing::info!("Registered observer: {} with worker", name);
        Ok(())
    }

    /// Enqueue a job for an observer by name
    /// Called by the bus subscriber when events are published
    pub async fn enqueue(&self, observer_name: &str, event: ExperienceEvent) -> Result<()> {
        let workers = self.workers.read().await;
        
        let handle = workers
            .get(observer_name)
            .ok_or_else(|| anyhow::anyhow!("No worker registered for observer: {}", observer_name))?;

        let job = ObserverJob::new(event);
        
        handle.sender
            .send(job)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue job: {}", e))?;

        tracing::debug!("Enqueued job for observer: {}", observer_name);
        Ok(())
    }

    /// Broadcast an event to all interested observers
    /// Checks each observer's accepts() before enqueueing
    pub async fn broadcast_event(&self, event: ExperienceEvent) -> Result<()> {
        let workers = self.workers.read().await;
        let event_type = event.event_type.clone();

        let mut enqueued_count = 0;
        
        for (name, handle) in workers.iter() {
            // We can't check accepts() here without the observer reference
            // So we send to all and let workers filter
            let job = ObserverJob::new(event.clone());
            if handle.sender.try_send(job).is_ok() {
                enqueued_count += 1;
            } else {
                tracing::warn!(
                    "Queue full for observer {}, dropping event of type {:?}",
                    name,
                    event_type
                );
            }
        }

        if enqueued_count > 0 {
            tracing::debug!("Broadcast event {:?} to {} observers", event_type, enqueued_count);
        }

        Ok(())
    }

    /// Get statistics for all workers
    pub async fn get_stats(&self) -> Vec<WorkerStats> {
        let workers = self.workers.read().await;
        workers.values()
            .map(|h| (*h.stats).clone())
            .collect()
    }

    /// Get statistics for a specific observer
    pub async fn get_observer_stats(&self, observer_name: &str) -> Option<WorkerStats> {
        let workers = self.workers.read().await;
        workers.get(observer_name).map(|h| (*h.stats).clone())
    }

    /// Get the number of registered workers
    pub async fn worker_count(&self) -> usize {
        let workers = self.workers.read().await;
        workers.len()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_manager_creation() {
        let bus = Arc::new(ExperienceBus::new());
        let manager = WorkerManager::new(bus);
        
        assert_eq!(manager.worker_count().await, 0);
    }
}
