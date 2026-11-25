//! Knowledge Graph Worker — Per Architecture Chapter 23.6.4.
//!
//! Handles knowledge graph-related background tasks: graph construction,
//! relationship updates, contradiction detection.

#[derive(Debug, Clone, Default)]
pub struct KnowledgeGraphWorker {
    /// Whether the worker is enabled.
    pub enabled: bool,
    /// Current priority.
    pub priority: String,
}
