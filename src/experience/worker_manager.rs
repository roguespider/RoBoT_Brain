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

        // Create the worker with sender so it can re-queue retries
        let worker = ExperienceWorker::new(observer, receiver, sender.clone());
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
    use crate::experience::observer::ExperienceObserver;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Test observer that tracks observed events
    struct TestObserver {
        name: &'static str,
        observed_count: Arc<AtomicU32>,
        accepted_events: Arc<AtomicU32>,
    }

    impl TestObserver {
        fn new(name: &'static str) -> (Self, Arc<AtomicU32>, Arc<AtomicU32>) {
            let observed_count = Arc::new(AtomicU32::new(0));
            let accepted_events = Arc::new(AtomicU32::new(0));
            let observer = Self {
                name,
                observed_count: observed_count.clone(),
                accepted_events: accepted_events.clone(),
            };
            (observer, observed_count, accepted_events)
        }
    }

    impl ExperienceObserver for TestObserver {
        fn name(&self) -> &'static str {
            self.name
        }

        fn accepts(&self, _event: &ExperienceEvent) -> bool {
            self.accepted_events.fetch_add(1, Ordering::SeqCst);
            true
        }

        fn observe(&self, _event: &ExperienceEvent) -> anyhow::Result<()> {
            self.observed_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_worker_manager_creation() {
        let bus = Arc::new(ExperienceBus::new());
        let manager = WorkerManager::new(bus);
        
        assert_eq!(manager.worker_count().await, 0);
    }

    #[tokio::test]
    async fn test_worker_manager_end_to_end() {
        let bus = Arc::new(ExperienceBus::new());
        let manager = Arc::new(WorkerManager::new(bus.clone()));

        let (observer, observed_count, accepted_count) = TestObserver::new("TestObserver");
        manager.register_observer(Arc::new(observer)).await.unwrap();

        let manager_clone = manager.clone();
        let bus_clone = bus.clone();
        let _handle = tokio::spawn(async move {
            let mut receiver = bus_clone.subscribe();
            while let Ok(event) = receiver.recv().await {
                let _ = manager_clone.broadcast_event(event).await;
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let experience = crate::experience::types::Experience::new(
            "Test Experience".to_string(),
            "A test experience for integration testing".to_string(),
            crate::experience::types::ExperienceType::ToolExecution,
            vec![],
        );
        let event = crate::experience::events::ExperienceEvent::experience_recorded(experience);
        bus.publish(event).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert!(accepted_count.load(Ordering::SeqCst) >= 1);
        assert!(observed_count.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn test_worker_manager_multiple_observers() {
        let bus = Arc::new(ExperienceBus::new());
        let manager = Arc::new(WorkerManager::new(bus.clone()));

        let (obs1, count1, _) = TestObserver::new("Observer1");
        let (obs2, count2, _) = TestObserver::new("Observer2");
        manager.register_observer(Arc::new(obs1)).await.unwrap();
        manager.register_observer(Arc::new(obs2)).await.unwrap();

        let manager_clone = manager.clone();
        let bus_clone = bus.clone();
        let _handle = tokio::spawn(async move {
            let mut receiver = bus_clone.subscribe();
            while let Ok(event) = receiver.recv().await {
                let _ = manager_clone.broadcast_event(event).await;
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let experience = crate::experience::types::Experience::new(
            "Test Experience".to_string(),
            "A test experience for multi-observer testing".to_string(),
            crate::experience::types::ExperienceType::ToolExecution,
            vec![],
        );
        let event = crate::experience::events::ExperienceEvent::experience_recorded(experience);
        bus.publish(event).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert!(count1.load(Ordering::SeqCst) >= 1);
        assert!(count2.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn test_worker_manager_get_stats() {
        let bus = Arc::new(ExperienceBus::new());
        let manager = Arc::new(WorkerManager::new(bus));

        let (observer, _, _) = TestObserver::new("StatsObserver");
        manager.register_observer(Arc::new(observer)).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].observer_name, "StatsObserver");
    }
}
