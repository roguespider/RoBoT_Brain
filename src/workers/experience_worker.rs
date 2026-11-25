//! Experience Worker — Per Architecture Chapter 23.6.2.
//!
//! Handles experience-related background tasks: scoring, pattern discovery,
//! skill development.

#[derive(Debug, Clone, Default)]
pub struct ExperienceWorker {
    /// Whether the worker is enabled.
    pub enabled: bool,
    /// Current priority.
    pub priority: String,
}
