use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

/// Global tool metrics collector for production use.
static GLOBAL_METRICS: LazyLock<Mutex<ToolMetrics>> =
    LazyLock::new(|| Mutex::new(ToolMetrics::new()));

/// Get the global metrics collector for telemetry.
/// Per Architecture §13.26 (Tool Health Monitoring): graceful degradation.
pub fn get_global_metrics() -> std::sync::MutexGuard<'static, ToolMetrics> {
    // Per Architecture §13.26 (Tool Health Monitoring): graceful degradation.
    // Use PoisonError::into_inner() — the idiomatic way to handle poisoned
    // mutexes without .unwrap(), .expect(), or _variable prefixes.
    match GLOBAL_METRICS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Tool execution metrics for observability.
/// Per Architecture §13.26 (Tool Health Monitoring): metrics collection.
pub struct ToolMetrics {
    /// Execution counts by tool.
    execution_counts: HashMap<String, u64>,
    /// Success counts by tool.
    success_counts: HashMap<String, u64>,
    /// Total execution time by tool.
    total_times: HashMap<String, Duration>,
    /// Last execution time by tool.
    last_times: HashMap<String, Duration>,
    /// Error counts by tool.
    error_counts: HashMap<String, u64>,
}

impl ToolMetrics {
    /// Create new metrics.
    pub fn new() -> Self {
        Self {
            execution_counts: HashMap::new(),
            success_counts: HashMap::new(),
            total_times: HashMap::new(),
            last_times: HashMap::new(),
            error_counts: HashMap::new(),
        }
    }

    /// Record a tool execution.
    pub fn record_execution(&mut self, tool_name: &str, duration: Duration, success: bool) {
        self.execution_counts
            .entry(tool_name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        if success {
            self.success_counts
                .entry(tool_name.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        } else {
            self.error_counts
                .entry(tool_name.to_string())
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        self.total_times
            .entry(tool_name.to_string())
            .and_modify(|t| *t += duration)
            .or_insert(duration);
        self.last_times
            .entry(tool_name.to_string())
            .and_modify(|t| *t = duration)
            .or_insert(duration);
    }

    /// Get execution count for a tool.
    pub fn get_execution_count(&self, tool_name: &str) -> u64 {
        self.execution_counts.get(tool_name).copied().unwrap_or(0)
    }

    /// Get success rate for a tool.
    pub fn get_success_rate(&self, tool_name: &str) -> f32 {
        let total = self.get_execution_count(tool_name);
        if total == 0 {
            return 0.0;
        }
        let success = self.success_counts.get(tool_name).copied().unwrap_or(0);
        success as f32 / total as f32
    }

    /// Get average execution time for a tool.
    pub fn get_average_time(&self, tool_name: &str) -> Duration {
        let total = self.get_execution_count(tool_name);
        if total == 0 {
            return Duration::from_millis(0);
        }
        let total_time = self
            .total_times
            .get(tool_name)
            .copied()
            .unwrap_or(Duration::from_millis(0));
        let total_ms = total_time.as_millis() as f64;
        let avg_ms = total_ms / total as f64;
        Duration::from_millis(avg_ms as u64)
    }

    /// Get error rate for a tool.
    pub fn get_error_rate(&self, tool_name: &str) -> f32 {
        let total = self.get_execution_count(tool_name);
        if total == 0 {
            return 0.0;
        }
        let errors = self.error_counts.get(tool_name).copied().unwrap_or(0);
        errors as f32 / total as f32
    }

    /// Get all tool metrics.
    pub fn get_all_metrics(&self) -> HashMap<String, ToolMetricSummary> {
        let mut result = HashMap::new();
        for (tool_name, _) in &self.execution_counts {
            result.insert(
                tool_name.clone(),
                ToolMetricSummary {
                    tool_name: tool_name.clone(),
                    execution_count: self.get_execution_count(tool_name),
                    success_rate: self.get_success_rate(tool_name),
                    average_time_ms: self.get_average_time(tool_name).as_millis() as u64,
                    error_rate: self.get_error_rate(tool_name),
                },
            );
        }
        result
    }
}

/// Summary of tool metrics.
#[derive(Debug, Clone)]
pub struct ToolMetricSummary {
    /// Tool name.
    pub tool_name: String,
    /// Total execution count.
    pub execution_count: u64,
    /// Success rate (0.0 to 1.0).
    pub success_rate: f32,
    /// Average execution time in milliseconds.
    pub average_time_ms: u64,
    /// Error rate (0.0 to 1.0).
    pub error_rate: f32,
}

/// Tool health monitor for health checks.
/// Per Architecture §13.25 (Tool Health Monitoring): health monitoring.
pub struct ToolHealthMonitor {
    /// Metrics collector.
    metrics: ToolMetrics,
    /// Health thresholds.
    health_thresholds: HealthThresholds,
}

impl ToolHealthMonitor {
    /// Create a new health monitor.
    pub fn new() -> Self {
        Self {
            metrics: ToolMetrics::new(),
            health_thresholds: HealthThresholds::default(),
        }
    }

    /// Check health of a tool.
    pub fn check_health(&self, tool_name: &str) -> ToolHealthStatus {
        let success_rate = self.metrics.get_success_rate(tool_name);
        let error_rate = self.metrics.get_error_rate(tool_name);
        let avg_time = self.metrics.get_average_time(tool_name);

        let mut status = ToolHealthStatus {
            tool_name: tool_name.to_string(),
            healthy: true,
            issues: Vec::new(),
        };

        if success_rate < self.health_thresholds.min_success_rate {
            status.healthy = false;
            status
                .issues
                .push(format!("Success rate too low: {:.2}", success_rate));
        }

        if error_rate > self.health_thresholds.max_error_rate {
            status.healthy = false;
            status
                .issues
                .push(format!("Error rate too high: {:.2}", error_rate));
        }

        if avg_time > self.health_thresholds.max_response_time {
            status.healthy = false;
            status.issues.push(format!(
                "Response time too high: {}ms",
                avg_time.as_millis()
            ));
        }

        status
    }

    /// Check health of all tools.
    pub fn check_all_health(&self) -> Vec<ToolHealthStatus> {
        let mut results = Vec::new();
        for tool_name in self.metrics.execution_counts.keys() {
            results.push(self.check_health(tool_name));
        }
        results
    }

    /// Update metrics with new execution data.
    pub fn record_execution(&mut self, tool_name: &str, duration: Duration, success: bool) {
        self.metrics.record_execution(tool_name, duration, success);
    }
}

/// Health thresholds for tool monitoring.
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// Minimum success rate (0.0 to 1.0).
    pub min_success_rate: f32,
    /// Maximum error rate (0.0 to 1.0).
    pub max_error_rate: f32,
    /// Maximum response time in milliseconds.
    pub max_response_time: Duration,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            min_success_rate: 0.8,
            max_error_rate: 0.2,
            max_response_time: Duration::from_millis(5000),
        }
    }
}

/// Health status of a tool.
#[derive(Debug, Clone)]
pub struct ToolHealthStatus {
    /// Tool name.
    pub tool_name: String,
    /// Whether the tool is healthy.
    pub healthy: bool,
    /// List of health issues.
    pub issues: Vec<String>,
}
