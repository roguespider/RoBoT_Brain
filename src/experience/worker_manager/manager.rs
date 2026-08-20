// /src/experience/worker_manager/manager.rs
//! Core WorkerManager implementation

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{RwLock, mpsc};

use chrono::Utc;
use uuid::Uuid;

use crate::experience::bus::ExperienceBus;
use crate::experience::events::ExperienceEvent;
use crate::experience::events::payload::EventPayload;
use crate::experience::events::types::ExperienceEventType;
use crate::experience::observer::ExperienceObserver;

use crate::experience::queue::JobQueue;
pub use crate::experience::worker::WorkerStats;
use crate::experience::worker::{
    ExperienceWorker, ObserverJob, OnCompleteCallback, OnFailedCallback, OnRetryCallback,
};

/// Maximum jobs that can be queued per observer
const MAX_QUEUE_DEPTH: usize = 100;

/// Handle to a worker's sender and stats
struct WorkerHandle {
    /// Sender to enqueue jobs for this worker
    sender: mpsc::Sender<ObserverJob>,
    /// Statistics for this worker
    stats: Arc<WorkerStats>,
}

/// In-memory registry mapping unique job IDs to their parent experience IDs.
/// This prevents ID collisions when multiple observers share the same event
/// (P0-002) and allows the worker callbacks to mark the correct job in the queue.
struct JobRegistry {
    /// Maps job_id -> experience_id
    mapping: Mutex<HashMap<String, String>>,
}

impl JobRegistry {
    fn new() -> Self {
        Self {
            mapping: Mutex::new(HashMap::new()),
        }
    }

    /// Register a job ID with its experience ID reference.
    fn register(&self, job_id: &str, experience_id: &str) {
        self.mapping
            .lock()
            .map(|mut m| {
                m.insert(job_id.to_string(), experience_id.to_string());
            })
            .ok();
    }

    /// Look up the experience ID for a given job ID.
    fn get_experience_id(&self, job_id: &str) -> Option<String> {
        self.mapping
            .lock()
            .ok()
            .and_then(|m| m.get(job_id).cloned())
    }
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
    /// Maps unique job IDs to experience IDs for tracking completion
    job_registry: Arc<JobRegistry>,
}

impl WorkerManager {
    /// Create a new worker manager with a durable SQLite-backed job queue.
    pub fn new_with_queue(bus: Arc<ExperienceBus>, job_queue: Arc<Mutex<JobQueue>>) -> Self {
        Self {
            bus,
            workers: Arc::new(RwLock::new(HashMap::new())),
            job_queue,
            job_registry: Arc::new(JobRegistry::new()),
        }
    }

    /// Register an observer and spawn a worker for it.
    /// The worker is configured with completion/failure callbacks that
    /// mark jobs in the durable queue.
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

        // Create callbacks that mark jobs in the durable queue.
        // Each closure needs its own clone of shared state since `move`
        // takes ownership of captured variables.
        let on_complete: OnCompleteCallback = {
            let jr = self.job_registry.clone();
            let jq = self.job_queue.clone();
            let on = name.clone();
            Arc::new(move |job_id: &str| {
                if let Some(exp_id) = jr.get_experience_id(job_id) {
                    tracing::debug!(
                        "Worker {} completed job {} (experience {})",
                        on,
                        job_id,
                        exp_id
                    );
                }
                let mut q = jq.lock().unwrap_or_else(|e| e.into_inner());
                q.mark_complete(job_id).unwrap_or_else(|e| {
                    tracing::warn!("Failed to mark job {} complete: {}", job_id, e);
                });
            })
        };

        let on_failed: OnFailedCallback = {
            let jr = self.job_registry.clone();
            let jq = self.job_queue.clone();
            let on = name.clone();
            Arc::new(move |job_id: &str, error: String| {
                if let Some(exp_id) = jr.get_experience_id(job_id) {
                    tracing::warn!(
                        "Worker {} permanently failed job {} (experience {}): {}",
                        on,
                        job_id,
                        exp_id,
                        error
                    );
                }
                let mut q = jq.lock().unwrap_or_else(|e| e.into_inner());
                q.mark_failed(job_id, error).unwrap_or_else(|e| {
                    tracing::warn!("Failed to mark job {} failed: {}", job_id, e);
                });
            })
        };

        // Retry callback: register the new retry job ID in the registry so
        // that the on_complete/on_failed callbacks can later find it
        // (P0-003: durable queue/worker state synchronization).
        let on_retry: OnRetryCallback = {
            let jr = self.job_registry.clone();
            Arc::new(move |new_job_id: &str, _original_job_id: &str| {
                jr.register(new_job_id, _original_job_id);
            })
        };

        // Create the worker with callbacks so it reports completion/failure/retry
        let worker = ExperienceWorker::with_callbacks(
            observer,
            receiver,
            sender.clone(),
            Some(on_complete),
            Some(on_failed),
            Some(on_retry),
        );
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

    /// Enqueue a job for an observer by name with a unique job ID.
    /// Called by the bus subscriber when events are published.
    /// The job is persisted to SQLite via the JobQueue.
    pub async fn enqueue(&self, observer_name: &str, event: ExperienceEvent) -> Result<()> {
        let workers = self.workers.read().await;

        let handle = workers.get(observer_name).ok_or_else(|| {
            anyhow::anyhow!("No worker registered for observer: {}", observer_name)
        })?;

        let event_id = event.id.to_string();
        let job_id = Uuid::new_v4().to_string();

        // Register the job ID -> experience ID mapping
        self.job_registry.register(&job_id, &event_id);

        // Persist the job to the durable queue before sending via channel
        {
            let mut q = self.job_queue.lock().unwrap_or_else(|e| e.into_inner());
            q.push_job_with_id(&job_id, &event_id, observer_name);
        }

        let job = ObserverJob::with_id(
            event,
            Uuid::parse_str(&job_id).unwrap_or_else(|_| Uuid::new_v4()),
        );

        handle
            .sender
            .send(job)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue job: {}", e))?;

        tracing::debug!("Enqueued job {} for observer: {}", job_id, observer_name);
        Ok(())
    }

    /// Broadcast an event to all interested observers.
    /// Each observer gets a unique job ID (P0-002). Jobs that fail to
    /// send (channel full) are marked as failed in the durable queue (P0-001).
    /// Jobs are persisted to the JobQueue for durability.
    ///
    /// Returns Ok(()) regardless of individual send failures - the dropped
    /// jobs are marked failed in the queue via the on_broadcast_failure callback.
    pub async fn broadcast_event(&self, event: ExperienceEvent) -> Result<()> {
        let workers = self.workers.read().await;
        let event_type = event.event_type.clone();
        let event_id = event.id.to_string();

        // Persist unique jobs to the durable queue for all workers
        let mut dropped_observers = Vec::new();

        {
            let mut q = self
                .job_queue
                .lock()
                .map_err(|e| anyhow::anyhow!("Queue lock poisoned: {}", e))?;
            for name in workers.keys() {
                let job_id = Uuid::new_v4().to_string();
                q.push_job_with_id(&job_id, &event_id, name);
                // Register the mapping so worker callbacks can find it
                self.job_registry.register(&job_id, &event_id);
                dropped_observers.push((name.clone(), job_id));
            }
        }

        let mut enqueued_count = 0;

        for (name, job_id) in dropped_observers {
            let handle = match workers.get(&name) {
                Some(h) => h,
                None => {
                    tracing::warn!("Observer {} disappeared during broadcast", name);
                    continue;
                }
            };

            let job = ObserverJob::with_id(
                event.clone(),
                Uuid::parse_str(&job_id).unwrap_or_else(|_| Uuid::new_v4()),
            );
            if handle.sender.try_send(job).is_ok() {
                enqueued_count += 1;
            } else {
                tracing::warn!(
                    "Queue full for observer {}, dropping event of type {:?}",
                    name,
                    event_type
                );
                // Mark the dropped job as failed in the durable queue
                if let Err(e) = self.mark_job_failed(
                    &job_id,
                    "Queue full - job dropped during broadcast".to_string(),
                ) {
                    tracing::debug!("Failed to mark dropped job {} as failed: {}", job_id, e);
                }
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

    /// Dispatch restored jobs back to their respective workers after a process
    /// restart. After `restore_from_database()` reloads jobs from SQLite, this
    /// method sends synthetic events through each worker's channel so that jobs
    /// that were in-flight when the process died get processed again.
    ///
    /// Registers each synthetic job ID in the `JobRegistry` so that the worker's
    /// completion callbacks can later find it (P0-003: durable queue/worker sync).
    pub async fn dispatch_restored_jobs(&self) {
        let q = self.job_queue.lock().unwrap_or_else(|e| e.into_inner());
        let jobs = q.pending_jobs();
        if jobs.is_empty() {
            return;
        }

        tracing::info!("Dispatching {} restored job(s) to workers", jobs.len());

        let workers = self.workers.read().await;

        for job in jobs {
            if let Some(handle) = workers.get(&job.observer_name) {
                // Register the synthetic job ID in the registry so worker
                // callbacks can look it up later (P0-003).
                self.job_registry.register(&job.id, &job.experience_id);

                let synthetic = ExperienceEvent {
                    id: Uuid::parse_str(&job.id).unwrap_or_else(|_| Uuid::new_v4()),
                    experience_id: Uuid::parse_str(&job.id).unwrap_or_else(|_| Uuid::new_v4()),
                    timestamp: Utc::now(),
                    event_type: ExperienceEventType::System,
                    payload: EventPayload::Experience {
                        experience_id: Uuid::parse_str(&job.id).unwrap_or_else(|_| Uuid::new_v4()),
                    },
                };
                let observer_job = ObserverJob::new(synthetic);
                if handle.sender.send(observer_job).await.is_err() {
                    tracing::warn!(
                        "Failed to dispatch restored job {} to observer {}: channel closed or full",
                        job.id,
                        job.observer_name
                    );
                } else {
                    tracing::debug!(
                        "Dispatched restored job {} to observer {}",
                        job.id,
                        job.observer_name
                    );
                }
            } else {
                tracing::warn!(
                    "No worker found for observer {} when dispatching restored job {}",
                    job.observer_name,
                    job.id
                );
            }
        }
    }
}
