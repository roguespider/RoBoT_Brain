//! Tool timeout handling — Per Architecture §13.14 (Tool Timeouts).
//! Provides timeout configuration and enforcement for tool execution.

use std::time::Duration;

/// Timeout configuration for tool execution.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Maximum execution time before timeout.
    pub execution_timeout: Duration,
    /// Grace period after timeout before forced termination.
    pub grace_period: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            execution_timeout: Duration::from_secs(30),
            grace_period: Duration::from_secs(5),
        }
    }
}

/// Tool timeout manager for enforcing execution deadlines.
/// Per Architecture §13.14: tools must not execute indefinitely.
pub struct ToolTimeout {
    config: TimeoutConfig,
}

impl ToolTimeout {
    /// Create a new timeout manager with the given configuration.
    pub fn new(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Create a timeout manager with default configuration.
    pub fn default_timeout() -> Self {
        Self::new(TimeoutConfig::default())
    }

    /// Get the execution timeout duration.
    pub fn execution_timeout(&self) -> Duration {
        self.config.execution_timeout
    }

    /// Get the grace period duration.
    pub fn grace_period(&self) -> Duration {
        self.config.grace_period
    }

    /// Get the total timeout (execution + grace).
    pub fn total_timeout(&self) -> Duration {
        self.config.execution_timeout + self.config.grace_period
    }

    /// Check if a duration has exceeded the execution timeout.
    pub fn is_expired(&self, elapsed: Duration) -> bool {
        elapsed >= self.config.execution_timeout
    }

    /// Check if a duration has exceeded the total timeout including grace.
    pub fn is_hard_expired(&self, elapsed: Duration) -> bool {
        elapsed >= self.total_timeout()
    }
}
