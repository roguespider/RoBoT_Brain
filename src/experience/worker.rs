// /src/experience/worker.rs
//! Background worker system for processing observer jobs per Architecture §22
//!
//! Design per README.md Pipeline Design:
//! Experience Recorded → Recorder → Bus → Job Queue → Workers → Observers

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::experience::{
    events::ExperienceEvent,
    observer::ExperienceObserver,
};

/// Maximum retry attempts for a failed job
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (in milliseconds)
const BASE_BACKOFF_MS: u64 = 100;

/// A job for the worker to process, with retry tracking
pub struct ObserverJob {
    /// The event to process
    pub event: ExperienceEvent,
    /// Number of retry attempts made
    pub retry_count: u32,
    /// Unique job ID for tracking
    pub job_id: Uuid,
}

impl ObserverJob {
    /// Create a new job from an event
    pub fn new(event: ExperienceEvent) -> Self {
        Self {
            event,
            retry_count: 0,
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
    stats: Arc<WorkerStats>,
}

impl ExperienceWorker {
    pub fn new(
        observer: Arc<dyn ExperienceObserver>,
        receiver: mpsc::Receiver<ObserverJob>,
    ) -> Self {
        let observer_name = observer.name().to_string();
        Self {
            observer,
            receiver,
            stats: Arc::new(WorkerStats::new(observer_name)),
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
                    tracing::debug!(
                        "Worker {} completed job {}",
                        observer_name,
                        job_id
                    );
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

    /// Handle a failed job with retry logic
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
        } else {
            tracing::error!(
                "Worker {} job {} permanently failed after {} attempts: {}",
                observer_name,
                job_id,
                MAX_RETRIES,
                error
            );
        }
    }
}
