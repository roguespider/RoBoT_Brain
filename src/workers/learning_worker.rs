//! Learning Worker — Per Architecture Chapter 23.6.3.
//!
//! Handles learning-related background tasks: hypothesis testing, knowledge
//! promotion, confidence updates.

#[derive(Debug, Clone, Default)]
pub struct LearningWorker {
    /// Whether the worker is enabled.
    pub enabled: bool,
    /// Current priority.
    pub priority: String,
}
