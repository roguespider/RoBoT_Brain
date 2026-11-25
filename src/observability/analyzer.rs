//! Observability Analyzer — Per Architecture Chapter 27.16 (Anomaly Detection).
//!
//! Observability enables detection of unusual behavior by comparing
//! current metrics against baselines.

/// An analyzer that detects anomalies in metrics.
pub struct Analyzer;

impl Analyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self
    }

    /// Detect whether a metric value represents an anomaly.
    pub fn is_anomaly(&self, current: f64, baseline: f64, threshold: f64) -> bool {
        let deviation = (current - baseline).abs();
        deviation > threshold
    }

    /// Compute the deviation ratio between current and baseline.
    pub fn deviation_ratio(&self, current: f64, baseline: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        (current - baseline).abs() / baseline
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
