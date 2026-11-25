//! Observability Timeline — Per Architecture Chapter 27.11 (Cognitive Timeline).
//!
//! RoBoT maintains a timeline of activity showing when events occurred
//! and in what order, enabling debugging of complex interactions.

/// A single entry in the cognitive timeline.
#[derive(Debug, Clone)]
pub struct TimelineEntry {
    /// Timestamp of the entry.
    pub timestamp: String,
    /// Description of what occurred.
    pub description: String,
}

/// A timeline of cognitive activity.
#[derive(Debug, Clone, Default)]
pub struct CognitiveTimeline {
    /// All timeline entries in chronological order.
    entries: Vec<TimelineEntry>,
}

impl CognitiveTimeline {
    /// Create a new empty timeline.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the timeline.
    pub fn add_entry(&mut self, timestamp: String, description: String) {
        self.entries.push(TimelineEntry {
            timestamp,
            description,
        });
    }

    /// Get all timeline entries.
    pub fn entries(&self) -> &[TimelineEntry] {
        &self.entries
    }

    /// Count total entries.
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}
