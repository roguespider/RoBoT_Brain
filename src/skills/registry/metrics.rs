// src/skills/registry/metrics.rs
//! Execution metrics for skills

use serde::{Deserialize, Serialize};

/// Execution metrics for a skill
/// Per Architecture §15: "Skill::track_execution_metrics()"
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_duration_ms: u64,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub avg_duration_ms: f64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
    pub last_failure: Option<chrono::DateTime<chrono::Utc>>,
    pub consecutive_successes: u64,
    pub consecutive_failures: u64,
}

impl ExecutionMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful execution
    pub fn record_success(&mut self, duration_ms: u64) {
        self.total_executions += 1;
        self.successful_executions += 1;
        self.total_duration_ms += duration_ms;
        self.avg_duration_ms = self.total_duration_ms as f64 / self.total_executions as f64;
        
        self.min_duration_ms = Some(
            self.min_duration_ms.map(|m| m.min(duration_ms)).unwrap_or(duration_ms)
        );
        self.max_duration_ms = Some(
            self.max_duration_ms.map(|m| m.max(duration_ms)).unwrap_or(duration_ms)
        );
        
        self.last_execution = Some(chrono::Utc::now());
        self.last_success = Some(chrono::Utc::now());
        self.consecutive_successes += 1;
        self.consecutive_failures = 0;
    }

    /// Record a failed execution
    pub fn record_failure(&mut self, duration_ms: u64) {
        self.total_executions += 1;
        self.failed_executions += 1;
        self.total_duration_ms += duration_ms;
        self.avg_duration_ms = self.total_duration_ms as f64 / self.total_executions as f64;
        
        self.min_duration_ms = Some(
            self.min_duration_ms.map(|m| m.min(duration_ms)).unwrap_or(duration_ms)
        );
        self.max_duration_ms = Some(
            self.max_duration_ms.map(|m| m.max(duration_ms)).unwrap_or(duration_ms)
        );
        
        self.last_execution = Some(chrono::Utc::now());
        self.last_failure = Some(chrono::Utc::now());
        self.consecutive_failures += 1;
        self.consecutive_successes = 0;
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f32 / self.total_executions as f32
        }
    }

    /// Get average duration in milliseconds
    pub fn avg_duration(&self) -> f64 {
        self.avg_duration_ms
    }

    /// Check if skill is stable (no recent failures)
    pub fn is_stable(&self) -> bool {
        self.consecutive_failures == 0 && self.successful_executions >= 3
    }

    /// Check if skill is unreliable (high failure rate)
    pub fn is_unreliable(&self) -> bool {
        self.total_executions >= 5 && self.success_rate() < 0.5
    }
}
