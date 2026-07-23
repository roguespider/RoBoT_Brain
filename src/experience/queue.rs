// /src/experience/queue.rs

#![allow(dead_code)]

use std::collections::HashMap;

/// Job status enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// A job in the queue
#[derive(Debug, Clone)]
pub struct Job {
    pub id: String,
    pub observer_name: String,
    pub status: JobStatus,
    pub last_error: Option<String>,
    pub attempts: u32,
}

impl Job {
    pub fn new(experience_id: &str, observer_name: &str) -> Self {
        Self {
            id: experience_id.to_string(),
            observer_name: observer_name.to_string(),
            status: JobStatus::Pending,
            last_error: None,
            attempts: 0,
        }
    }
}

/// A simple in-memory job queue.
/// In a production system, this could be backed by SQLite or Redis.
pub struct JobQueue {
    jobs: HashMap<String, Job>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    /// Add a new job to the queue
    pub fn push_job(&mut self, experience_id: &str, observer_name: &str) {
        let job = Job::new(experience_id, observer_name);
        self.jobs.insert(job.id.clone(), job);
    }

    /// Get the next pending job for a specific observer
    pub fn pop_job(&mut self, observer_name: &str) -> Option<Job> {
        // Find a job that matches the observer and is Pending
        let job_id = self
            .jobs
            .iter()
            .find(|(_, job)| job.observer_name == observer_name && job.status == JobStatus::Pending)
            .map(|(id, _)| id.clone());

        if let Some(id) = job_id {
            let job = self.jobs.get(&id)
                .expect("Job ID from iterator should always exist")
                .clone();
            // Mark as running immediately to prevent race conditions
            self.mark_running(&id);
            Some(job)
        } else {
            None
        }
    }

    /// Mark a job as completed
    pub fn complete_job(&mut self, job_id: &str) {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Completed;
        }
    }

    /// Mark a job as failed
    pub fn fail_job(&mut self, job_id: &str, error: String) {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Failed;
            job.last_error = Some(error);
            job.attempts += 1;
        }
    }

    /// Helper to mark as running (internal use)
    fn mark_running(&mut self, job_id: &str) {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Running;
        }
    }
}
