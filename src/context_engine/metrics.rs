//! Context metrics — Per Architecture §07 (Context Lifecycle).
//! Tracks context assembly and retrieval metrics.

use std::collections::HashMap;
use std::time::Duration;

/// Context metrics collector.
pub struct ContextMetrics {
    /// Assembly count by context ID.
    assembly_counts: HashMap<String, u64>,
    /// Total assembly time by context ID.
    total_times: HashMap<String, Duration>,
}

impl ContextMetrics {
    /// Create a new context metrics collector.
    pub fn new() -> Self {
        Self {
            assembly_counts: HashMap::new(),
            total_times: HashMap::new(),
        }
    }

    /// Record a context assembly.
    pub fn record_assembly(&mut self, context_id: &str, duration: Duration) {
        self.assembly_counts
            .entry(context_id.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        self.total_times
            .entry(context_id.to_string())
            .and_modify(|t| *t += duration)
            .or_insert(duration);
    }

    /// Get assembly count for a context.
    pub fn get_assembly_count(&self, context_id: &str) -> u64 {
        self.assembly_counts.get(context_id).copied().unwrap_or(0)
    }

    /// Get total assembly time for a context.
    pub fn get_total_time(&self, context_id: &str) -> Duration {
        self.total_times.get(context_id).copied().unwrap_or(Duration::from_millis(0))
    }
}

impl Default for ContextMetrics {
    fn default() -> Self {
        Self::new()
    }
}
