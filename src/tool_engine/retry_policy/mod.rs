// src/tools/retry_policy/mod.rs
//! Retry policy for tool execution.
//!
//! Per Architecture §13.23: tools declare retry characteristics including
//! max retries, backoff strategy, and which errors are retryable.

/// Retry backoff strategy.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries.
    Fixed { delay_ms: u64 },
    /// Exponential backoff with jitter.
    Exponential {
        base_delay_ms: u64,
        max_delay_ms: u64,
        jitter: bool,
    },
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Fixed { delay_ms: 1000 }
    }
}

/// Retry policy for a tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Backoff strategy between retries.
    pub backoff: BackoffStrategy,
    /// Which error types are retryable.
    pub retryable_errors: Vec<String>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::default(),
            retryable_errors: vec!["timeout".to_string(), "rate_limit".to_string()],
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with the given max retries.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Check if an error is retryable.
    pub fn is_retryable(&self, error_name: &str) -> bool {
        self.retryable_errors.iter().any(|e| e == error_name)
    }
}
