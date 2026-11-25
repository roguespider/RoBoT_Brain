//! Memory Worker — Per Architecture Chapter 23.6.1.
//!
//! Handles memory-related background tasks: consolidation, promotion, forgetting.

#[derive(Debug, Clone, Default)]
pub struct MemoryWorker {
    /// Whether the worker is enabled.
    pub enabled: bool,
    /// Current priority.
    pub priority: String,
}
