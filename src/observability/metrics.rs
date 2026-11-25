//! Observability Metrics — Per Architecture Chapter 27.5 (System Metrics).
//!
//! System metrics measure technical health including CPU, memory, GPU,
//! database performance, worker status, and response latency.

/// A system metric value.
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name.
    pub name: String,
    /// Metric value.
    pub value: f64,
    /// Unit of measurement.
    pub unit: String,
    /// Timestamp of the measurement.
    pub timestamp: i64,
}

/// Collection of system metrics.
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// All recorded metrics.
    values: Vec<Metric>,
}

impl Metrics {
    /// Create a new empty metrics collection.
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Record a metric value.
    pub fn record(&mut self, name: String, value: f64, unit: String, timestamp: i64) {
        self.values.push(Metric {
            name,
            value,
            unit,
            timestamp,
        });
    }

    /// Get all metrics.
    pub fn all(&self) -> &[Metric] {
        &self.values
    }

    /// Get metrics by name.
    pub fn by_name(&self, name: &str) -> Vec<&Metric> {
        self.values.iter().filter(|m| m.name == name).collect()
    }

    /// Count total metrics.
    pub fn count(&self) -> usize {
        self.values.len()
    }
}
