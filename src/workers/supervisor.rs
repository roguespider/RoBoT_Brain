//! Worker Supervisor — Per Architecture Chapter 23.4.
//!
//! Re-exports the `WorkerSupervisor` from the scheduler module,
//! which manages worker lifecycle: register → schedule → execute → monitor → recover.

pub use crate::scheduler::WorkerSupervisor;

/// Status of a background worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    /// Worker is running.
    Running,
    /// Worker is idle.
    Idle,
    /// Worker is processing a task.
    Processing,
    /// Worker has failed.
    Failed,
    /// Worker is being restarted.
    Restarting,
}
