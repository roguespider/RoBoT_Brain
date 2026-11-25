//! Retrieval telemetry per Architecture Chapter 16.
//! Records metrics and traces for retrieval operations.

/// Telemetry metrics for a single retrieval operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrievalTelemetry {
    /// Query that was executed.
    pub query: String,
    /// Duration of the retrieval in milliseconds.
    pub duration_ms: u64,
    /// Number of candidates found.
    pub candidates_found: usize,
    /// Number of candidates returned after filtering.
    pub candidates_returned: usize,
    /// Average confidence of returned candidates.
    pub avg_confidence: f32,
    /// Stages that were executed.
    pub stages_executed: Vec<String>,
}

/// Collector for retrieval telemetry across multiple operations.
pub struct RetrievalTelemetryCollector {
    records: Vec<RetrievalTelemetry>,
}

impl RetrievalTelemetryCollector {
    /// Create a new empty collector.
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Record a retrieval telemetry entry.
    pub fn record(&mut self, telemetry: RetrievalTelemetry) {
        self.records.push(telemetry);
    }

    /// Get the average duration across all recorded operations.
    pub fn average_duration_ms(&self) -> f32 {
        if self.records.is_empty() {
            return 0.0;
        }
        let total: u64 = self.records.iter().map(|r| r.duration_ms).sum();
        total as f32 / self.records.len() as f32
    }

    /// Get the total number of recorded operations.
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// Get the average number of candidates returned per operation.
    pub fn average_candidates_returned(&self) -> f32 {
        if self.records.is_empty() {
            return 0.0;
        }
        let total: usize = self
            .records
            .iter()
            .map(|r| r.candidates_returned)
            .sum();
        total as f32 / self.records.len() as f32
    }
}

impl Default for RetrievalTelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}
