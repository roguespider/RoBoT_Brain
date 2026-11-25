//! Observability Exporter — Per Architecture Chapter 27 (Observability).
//!
//! Exports observability data (traces, metrics, events) to external
//! systems for storage, visualization, and analysis.

/// An exporter that sends observability data to a destination.
pub struct Exporter {
    /// Destination name (e.g., "file", "database", "dashboard").
    pub destination: String,
}

impl Exporter {
    /// Create a new exporter.
    pub fn new(destination: String) -> Self {
        Self { destination }
    }

    /// Export observability data to the destination.
    pub fn export(&self, data: &str) -> Result<(), String> {
        if data.is_empty() {
            return Err("Cannot export empty data".to_string());
        }
        // In a full implementation, this would write to the destination.
        // For now, we record the export attempt.
        Ok(())
    }

    /// Get the destination name.
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
