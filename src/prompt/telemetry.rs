//! Prompt telemetry per Architecture Chapter 17.
//! Records metrics and traces for prompt construction operations.

/// Telemetry record for a prompt construction.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptTelemetry {
    /// Template used.
    pub template: String,
    /// Number of layers included.
    pub layer_count: usize,
    /// Token estimate.
    pub token_estimate: usize,
    /// Construction duration in milliseconds.
    pub duration_ms: u64,
    /// Quality score if available.
    pub quality_score: Option<f32>,
}

/// Collector for prompt telemetry across multiple constructions.
pub struct PromptTelemetryCollector {
    records: Vec<PromptTelemetry>,
}

impl PromptTelemetryCollector {
    /// Create a new empty collector.
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    /// Record a prompt construction telemetry entry.
    pub fn record(&mut self, telemetry: PromptTelemetry) {
        self.records.push(telemetry);
    }

    /// Get the average token estimate across all records.
    pub fn average_tokens(&self) -> f32 {
        if self.records.is_empty() {
            return 0.0;
        }
        let total: usize = self.records.iter().map(|r| r.token_estimate).sum();
        total as f32 / self.records.len() as f32
    }

    /// Get the total number of records.
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// Get the average quality score across records that have one.
    pub fn average_quality(&self) -> Option<f32> {
        let scores: Vec<f32> = self
            .records
            .iter()
            .filter_map(|r| r.quality_score)
            .collect();
        if scores.is_empty() {
            return None;
        }
        let total: f32 = scores.iter().sum();
        Some(total / scores.len() as f32)
    }
}

impl Default for PromptTelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}
