// /src/experience/queue.rs
//
// Job queue for the experience worker subsystem (Architecture §23.5 Task
// Queue).
//
// The queue is durable when a `SqliteDatabase` is supplied: every operation
// is mirrored to the `job_queue` table (migration 012) so pending jobs
// survive a process restart. When no database is supplied it falls back to
// an in-memory `HashMap`, preserving the original behavior used by the
// startup self-check probe.

use std::collections::HashMap;

use anyhow::Result;

use crate::database::sqlite::SqliteDatabase;

/// Job status enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl JobStatus {
    fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(JobStatus::Pending),
            "running" => Some(JobStatus::Running),
            "completed" => Some(JobStatus::Completed),
            "failed" => Some(JobStatus::Failed),
            _ => None,
        }
    }
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

/// A job queue backed by SQLite when a database is configured, falling back
/// to an in-memory map otherwise.
pub struct JobQueue {
    /// In-memory cache used when no database is configured.
    jobs: HashMap<String, Job>,
    /// Optional durable store. When set, every op is mirrored to SQLite.
    database: Option<Arc<SqliteDatabase>>,
}

use std::sync::Arc;

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            database: None,
        }
    }

    /// Create a durable queue backed by the given database.
    pub fn with_database(database: Arc<SqliteDatabase>) -> Self {
        Self {
            jobs: HashMap::new(),
            database: Some(database),
        }
    }

    /// Add a new job to the queue
    pub fn push_job(&mut self, experience_id: &str, observer_name: &str) {
        let job = Job::new(experience_id, observer_name);
        if let Some(db) = &self.database {
            if let Err(e) = persist_insert(db, &job) {
                tracing::warn!("JobQueue insert failed, job not durable: {}", e);
            }
        }
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
            // Safety: job_id came from iterating over self.jobs, so get must succeed
            let job = match self.jobs.get(&id) {
                Some(j) => j.clone(),
                None => {
                    // Unreachable - id came from iterator over self.jobs
                    tracing::error!("Unexpected: job ID not found in jobs map");
                    return None;
                }
            };
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
            if let Some(db) = &self.database {
                if let Err(e) = persist_update(db, job) {
                    tracing::warn!("JobQueue complete persist failed: {}", e);
                }
            }
        }
    }

    /// Mark a job as failed
    pub fn fail_job(&mut self, job_id: &str, error: String) {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Failed;
            job.last_error = Some(error);
            job.attempts += 1;
            if let Some(db) = &self.database {
                if let Err(e) = persist_update(db, job) {
                    tracing::warn!("JobQueue fail persist failed: {}", e);
                }
            }
        }
    }

    /// Helper to mark as running (internal use)
    fn mark_running(&mut self, job_id: &str) {
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Running;
            if let Some(db) = &self.database {
                if let Err(e) = persist_update(db, job) {
                    tracing::warn!("JobQueue running persist failed: {}", e);
                }
            }
        }
    }

    /// Mark a job as completed (updates in-memory cache and SQLite).
    pub fn mark_complete(&mut self, job_id: &str) -> Result<()> {
        let db = self.database.as_ref();
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Completed;
        }
        if let Some(db) = db {
            if let Err(e) = persist_update(
                db,
                &Job {
                    id: job_id.to_string(),
                    observer_name: String::new(),
                    status: JobStatus::Completed,
                    last_error: None,
                    attempts: 0,
                },
            ) {
                tracing::warn!("JobQueue mark_complete persist failed: {}", e);
            }
        }
        Ok(())
    }

    /// Mark a job as failed with an error message (updates in-memory cache and SQLite).
    pub fn mark_failed(&mut self, job_id: &str, error: String) -> Result<()> {
        let db = self.database.as_ref();
        if let Some(job) = self.jobs.get_mut(job_id) {
            job.status = JobStatus::Failed;
            job.last_error = Some(error.clone());
            job.attempts += 1;
        }
        if let Some(db) = db {
            if let Err(e) = persist_update(
                db,
                &Job {
                    id: job_id.to_string(),
                    observer_name: String::new(),
                    status: JobStatus::Failed,
                    last_error: Some(error),
                    attempts: 1,
                },
            ) {
                tracing::warn!("JobQueue mark_failed persist failed: {}", e);
            }
        }
        Ok(())
    }

    /// Reload pending/running jobs from SQLite into the in-memory cache.
    ///
    /// Called at startup so a durable queue resumes work that was in flight
    /// before the process restarted. Running jobs are demoted to Pending so
    /// a worker picks them up again (no other worker can have claimed them
    /// in this single-process model).
    pub fn restore_from_database(&mut self) -> Result<usize> {
        let db = match &self.database {
            Some(db) => db,
            None => return Ok(0),
        };
        let conn = db.connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, observer_name, status, last_error, attempts
               FROM job_queue
              WHERE status IN ('pending', 'running')",
        )?;
        let rows: Vec<(String, String, String, Option<String>, u32)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, u32>(4)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut restored = 0usize;
        for (id, observer_name, status_str, last_error, attempts) in rows {
            // Demote any 'running' jobs left over from a crash to 'pending'.
            let mut status = JobStatus::from_str(&status_str).unwrap_or(JobStatus::Pending);
            if status == JobStatus::Running {
                status = JobStatus::Pending;
                if let Err(e) = conn.execute(
                    "UPDATE job_queue SET status = 'pending', updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![now_iso(), &id],
                ) {
                    tracing::warn!("JobQueue restore demotion failed for {}: {}", id, e);
                }
            }
            let job = Job {
                id: id.clone(),
                observer_name,
                status,
                last_error,
                attempts,
            };
            self.jobs.insert(id, job);
            restored += 1;
        }
        Ok(restored)
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ==========================================================
// SQLite persistence helpers
// ==========================================================

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

fn persist_insert(db: &SqliteDatabase, job: &Job) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "INSERT OR REPLACE INTO job_queue
            (id, observer_name, status, last_error, attempts, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            &job.id,
            &job.observer_name,
            job.status.as_str(),
            job.last_error.as_deref(),
            job.attempts,
            now_iso(),
            now_iso(),
        ],
    )?;
    Ok(())
}

fn persist_update(db: &SqliteDatabase, job: &Job) -> Result<()> {
    let conn = db.connection()?;
    conn.execute(
        "UPDATE job_queue
            SET status = ?1, last_error = ?2, attempts = ?3, updated_at = ?4
          WHERE id = ?5",
        rusqlite::params![
            job.status.as_str(),
            job.last_error.as_deref(),
            job.attempts,
            now_iso(),
            &job.id,
        ],
    )?;
    Ok(())
}
