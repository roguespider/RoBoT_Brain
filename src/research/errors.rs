use std::fmt;

/// Research errors with structured information for better error handling
#[derive(Debug)]
pub enum ResearchError {
    /// Timeout occurred during search operation
    Timeout {
        query: String,
        elapsed: std::time::Duration,
    },
    /// Search provider is unavailable
    ProviderUnavailable { provider: String },
    /// Search provider returned an error
    ProviderError { provider: String, message: String },
    /// No results found for the query
    NoResults { query: String },
    /// Failed to extract content from a source
    ContentExtractionFailed { url: String },
    /// Operation was cancelled
    Cancelled,
}

impl fmt::Display for ResearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResearchError::Timeout { query, elapsed } => {
                write!(f, "Timeout after {:?} for query: {}", elapsed, query)
            }
            ResearchError::ProviderUnavailable { provider } => {
                write!(f, "Provider '{}' is unavailable", provider)
            }
            ResearchError::ProviderError { provider, message } => {
                write!(f, "Provider '{}' error: {}", provider, message)
            }
            ResearchError::NoResults { query } => {
                write!(f, "No results found for query: {}", query)
            }
            ResearchError::ContentExtractionFailed { url } => {
                write!(f, "Failed to extract content from URL: {}", url)
            }
            ResearchError::Cancelled => {
                write!(f, "Operation was cancelled")
            }
        }
    }
}

impl std::error::Error for ResearchError {}

impl From<anyhow::Error> for ResearchError {
    fn from(e: anyhow::Error) -> Self {
        ResearchError::ProviderError {
            provider: "anyhow".to_string(),
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "http")]
impl From<reqwest::Error> for ResearchError {
    fn from(e: reqwest::Error) -> Self {
        ResearchError::ProviderError {
            provider: "reqwest".to_string(),
            message: e.to_string(),
        }
    }
}

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn with_timeout<F, T>(fut: F, dur: std::time::Duration) -> Result<T, ResearchError>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(dur, fut)
        .await
        .map_err(|_| ResearchError::Timeout {
            query: "timeout".to_string(),
            elapsed: dur,
        })
}

pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
