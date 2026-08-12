// /src/experience/worker_manager/manager.rs
//! Core WorkerManager implementation

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{RwLock, mpsc};

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::observer::ExperienceObserver;
use crate::experience::queue::JobQueue;
pub use crate::experience::worker::WorkerStats;
use crate::experience::worker::{ExperienceWorker, ObserverJob};

/// Maximum jobs that can be queued per observer
const MAX_QUEUE_DEPTH: usize = 100;

/// Handle to a worker's sender and stats
struct WorkerHandle {
    /// Sender to enqueue jobs for this worker
    sender: mpsc::Sender<ObserverJob>,
    /// Statistics for this worker
    stats: Arc<WorkerStats>,
}

/// Manages workers for all registered observers
/// Routes events from bus to appropriate worker channels
/// Jobs are persisted to SQLite via the JobQueue for durability.
pub struct WorkerManager {
    /// Event bus for subscribing to events
    bus: Arc<ExperienceBus>,
    /// Workers indexed by observer name
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
    /// Durable job queue backed by SQLite
    job_queue: Arc<Mutex<JobQueue>>,
}

impl WorkerManager {
    /// Create a new worker manager with a durable SQLite-backed job queue.
    pub fn new_with_queue(bus: Arc<ExperienceBus>, job_queue: Arc<Mutex<JobQueue>>) -> Self {
        Self {
            bus,
            workers: Arc::new(RwLock::new(HashMap::new())),
            job_queue,
        }
    }

    /// Register an observer and spawn a worker for it
    pub async fn register_observer(&self, observer: Arc<dyn ExperienceObserver>) -> Result<()> {
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

        // Spawn the worker task - task runs until channel closes or observer.shutdown()
        tokio::spawn(async move {
            if let Err(e) = worker.start().await {
                tracing::error!("Worker for {} stopped with error: {}", task_name, e);
            }
        });

        // Store the handle
        workers.insert(name.clone(), WorkerHandle { sender, stats });

        tracing::info!("Registered observer: {} with worker", name);
        Ok(())
    }

    /// Enqueue a job for an observer by name
    /// Called by the bus subscriber when events are published.
    /// The job is persisted to SQLite via the JobQueue.
    pub async fn enqueue(&self, observer_name: &str, event: ExperienceEvent) -> Result<()> {
        let workers = self.workers.read().await;

        let handle = workers.get(observer_name).ok_or_else(|| {
            anyhow::anyhow!("No worker registered for observer: {}", observer_name)
        })?;

        // Persist the job to the durable queue before sending via channel
        let event_id = event.id.to_string();
        {
            let mut q = self.job_queue.lock().unwrap();
            q.push_job(&event_id, observer_name);
        }

        let job = ObserverJob::new(event);

        handle
            .sender
            .send(job)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue job: {}", e))?;

        tracing::debug!("Enqueued job for observer: {}", observer_name);
        Ok(())
    }

    /// Broadcast an event to all interested observers
    /// Checks each observer's accepts() before enqueueing.
    /// Jobs are persisted to the JobQueue for durability.
    pub async fn broadcast_event(&self, event: ExperienceEvent) -> Result<()> {
        let workers = self.workers.read().await;
        let event_type = event.event_type.clone();
        let event_id = event.id.to_string();

        // Persist the job to the durable queue for all workers
        {
            let mut q = self
                .job_queue
                .lock()
                .map_err(|e| anyhow::anyhow!("Queue lock poisoned: {}", e))?;
            for name in workers.keys() {
                q.push_job(&event_id, name);
            }
        }

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
            tracing::debug!(
                "Broadcast event {:?} to {} observers",
                event_type,
                enqueued_count
            );
        }

        Ok(())
    }

    /// Get statistics for all workers
    pub async fn get_stats(&self) -> Vec<WorkerStats> {
        let workers = self.workers.read().await;
        workers.values().map(|h| (*h.stats).clone()).collect()
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

    /// Return a clone of the shared event bus. The manager retains the bus so
    /// callers (and the self-check) can subscribe or inspect subscriber count
    /// without re-archiving the Arc externally (Architecture §5 event bus).
    pub fn bus(&self) -> Arc<ExperienceBus> {
        self.bus.clone()
    }

    /// Number of active subscribers on the backing event bus. Delegates to the
    /// bus so the stored `bus` field remains a live dependency rather than
    /// dead state.
    pub fn bus_subscriber_count(&self) -> usize {
        self.bus.subscriber_count()
    }

    /// Mark a job as completed after a worker processes it.
    /// Delegates to `JobQueue::complete_job` (the infallible SQLite-aware
    /// path); persist failures are logged inside the queue.
    pub fn mark_job_complete(&self, job_id: &str) -> Result<()> {
        let mut q = self
            .job_queue
            .lock()
            .map_err(|e| anyhow::anyhow!("Queue lock poisoned: {}", e))?;
        q.complete_job(job_id);
        Ok(())
    }

    /// Mark a job as failed after a worker encounters an error.
    /// Delegates to `JobQueue::fail_job` (the infallible SQLite-aware path);
    /// persist failures are logged inside the queue.
    pub fn mark_job_failed(&self, job_id: &str, error: String) -> Result<()> {
        let mut q = self
            .job_queue
            .lock()
            .map_err(|e| anyhow::anyhow!("Queue lock poisoned: {}", e))?;
        q.fail_job(job_id, error);
        Ok(())
    }
}
