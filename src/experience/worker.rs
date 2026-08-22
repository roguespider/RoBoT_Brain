// /src/experience/worker.rs
//! Background worker system for processing observer jobs per Architecture §22
//!
//! Design per README.md Pipeline Design:
//! Experience Recorded → Recorder → Bus → Job Queue → Workers → Observers

use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::experience::{events::ExperienceEvent, observer::ExperienceObserver};

/// Maximum retry attempts for a failed job
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (in milliseconds)
const BASE_BACKOFF_MS: u64 = 100;

/// Callback type for job completion notifications
pub type OnCompleteCallback = Arc<dyn Fn(&str) + Send + Sync>;
/// Callback type for job failure notifications
pub type OnFailedCallback = Arc<dyn Fn(&str, String) + Send + Sync>;
/// Callback type for job retry notifications (new_job_id, original_job_id)
pub type OnRetryCallback = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// A job for the worker to process, with retry tracking
#[derive(Debug, Clone)]
pub struct ObserverJob {
    /// The event to process
    pub event: ExperienceEvent,
    /// Number of retry attempts made
    pub retry_count: u32,
    /// Unique job ID for tracking
    pub job_id: Uuid,
}

impl ObserverJob {
    /// Create a new job from an event, generating a unique job ID.
    pub fn new(event: ExperienceEvent) -> Self {
        Self {
            event,
            retry_count: 0,
            job_id: Uuid::new_v4(),
        }
    }

    /// Create a new job from an event using a pre-generated job ID.
    /// Use this when the caller needs the job ID to match a durable
    /// queue entry (P0-002: Unique Durable Job Identity).
    pub fn with_id(event: ExperienceEvent, job_id: Uuid) -> Self {
        Self {
            event,
            retry_count: 0,
            job_id,
        }
    }

    /// Create a retry job with incremented retry count.
    /// Generates a new unique job ID for this retry attempt.
    pub fn with_retry(event: ExperienceEvent, original_job: &ObserverJob) -> Self {
        Self {
            event,
            retry_count: original_job.retry_count + 1,
            job_id: Uuid::new_v4(),
        }
    }
}

/// Statistics for a worker's activity
#[derive(Debug)]
pub struct WorkerStats {
    pub observer_name: String,
    pub jobs_processed: AtomicU64,
    pub jobs_failed: AtomicU64,
    pub jobs_retried: AtomicU64,
}

impl WorkerStats {
    pub fn new(observer_name: String) -> Self {
        Self {
            observer_name,
            jobs_processed: AtomicU64::new(0),
            jobs_failed: AtomicU64::new(0),
            jobs_retried: AtomicU64::new(0),
        }
    }
}

impl Clone for WorkerStats {
    fn clone(&self) -> Self {
        Self {
            observer_name: self.observer_name.clone(),
            jobs_processed: AtomicU64::new(self.jobs_processed.load(Ordering::SeqCst)),
            jobs_failed: AtomicU64::new(self.jobs_failed.load(Ordering::SeqCst)),
            jobs_retried: AtomicU64::new(self.jobs_retried.load(Ordering::SeqCst)),
        }
    }
}

/// Worker that processes jobs for a specific observer
pub struct ExperienceWorker {
    observer: Arc<dyn ExperienceObserver>,
    receiver: mpsc::Receiver<ObserverJob>,
    sender: mpsc::Sender<ObserverJob>,
    stats: Arc<WorkerStats>,
    /// Called when a job completes successfully (with job_id)
    on_complete: Option<OnCompleteCallback>,
    /// Called when a job permanently fails after max retries (with job_id, error)
    on_failed: Option<OnFailedCallback>,
    /// Called when a retry job is created (new_job_id, original_job_id)
    /// Used to register the retry's new ID in the JobRegistry so the
    /// worker callbacks can later find it (P0-003 synchronization).
    on_retry: Option<OnRetryCallback>,
}

impl ExperienceWorker {
    /// Create a worker with completion/failure/retry callbacks.
    /// The callbacks receive the unique job_id and are invoked when a job
    /// completes successfully, fails permanently (after all retries exhausted),
    /// or when a retry job is created (P0-003: durable queue/worker sync).
    pub fn with_callbacks(
        observer: Arc<dyn ExperienceObserver>,
        receiver: mpsc::Receiver<ObserverJob>,
        sender: mpsc::Sender<ObserverJob>,
        on_complete: Option<OnCompleteCallback>,
        on_failed: Option<OnFailedCallback>,
        on_retry: Option<OnRetryCallback>,
    ) -> Self {
        let observer_name = observer.name().to_string();
        Self {
            observer,
            receiver,
            sender,
            stats: Arc::new(WorkerStats::new(observer_name)),
            on_complete,
            on_failed,
            on_retry,
        }
    }

    /// Get worker statistics
    pub fn stats(&self) -> Arc<WorkerStats> {
        self.stats.clone()
    }

    /// Start the worker - runs until channel closes
    /// Processes jobs from the queue and calls observer.observe()
    pub async fn start(mut self) -> Result<()> {
        self.observer.start()?;

        tracing::info!("Worker started for observer: {}", self.observer.name());

        while let Some(job) = self.receiver.recv().await {
            let observer_name = self.observer.name().to_string();
            let job_id = job.job_id;

            // Check if observer accepts this event
            if !self.observer.accepts(&job.event) {
                tracing::debug!(
                    "Worker {} skipping event {} (not accepted)",
                    observer_name,
                    job_id
                );
                continue;
            }

            // Process the job
            match self.observer.observe(&job.event) {
                Ok(_) => {
                    self.stats.jobs_processed.fetch_add(1, Ordering::SeqCst);
                    tracing::debug!("Worker {} completed job {}", observer_name, job_id);
                    if let Some(cb) = &self.on_complete {
                        let job_id_str = job_id.to_string();
                        cb(&job_id_str);
                    }
                }
                Err(err) => {
                    self.handle_failure(job, err.to_string()).await;
                }
            }
        }

        self.observer.shutdown()?;
        tracing::info!("Worker stopped for observer: {}", self.observer.name());

        Ok(())
    }

    /// Handle a failed job with retry logic - re-queues with backoff if retries remaining
    async fn handle_failure(&self, job: ObserverJob, error: String) {
        let observer_name = self.observer.name().to_string();
        let job_id = job.job_id;

        self.stats.jobs_failed.fetch_add(1, Ordering::SeqCst);

        if job.retry_count < MAX_RETRIES {
            // Calculate exponential backoff delay
            let delay_ms = BASE_BACKOFF_MS * 2u64.pow(job.retry_count);

            tracing::warn!(
                "Worker {} job {} failed (attempt {}/{}): {}. Retrying in {}ms",
                observer_name,
                job_id,
                job.retry_count + 1,
                MAX_RETRIES,
                error,
                delay_ms
            );

            self.stats.jobs_retried.fetch_add(1, Ordering::SeqCst);

            // Create retry job with incremented retry count
            let retry_job = ObserverJob::with_retry(job.event.clone(), &job);

            // Register the retry's new job ID so the durable queue callbacks
            // can find it later (P0-003: worker/state synchronization).
            if let Some(cb) = &self.on_retry {
                let new_id = retry_job.job_id.to_string();
                cb(&new_id, &job.job_id.to_string());
            }

            // Wait for backoff delay then re-queue
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            if let Err(e) = self.sender.send(retry_job).await {
                tracing::error!(
                    "Worker {} failed to re-queue job {} after retry: {}",
                    observer_name,
                    job_id,
                    e
                );
            }
        } else {
            tracing::error!(
                "Worker {} job {} permanently failed after {} attempts: {}",
                observer_name,
                job_id,
                MAX_RETRIES,
                error
            );
            if let Some(cb) = &self.on_failed {
                let job_id_str = job_id.to_string();
                cb(&job_id_str, error);
            }
        }
    }
}
