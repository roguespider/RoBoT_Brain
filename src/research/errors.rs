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
    // tokio::sync::oneshot for cancellation signal (Architecture §R9)
    cancel_tx: std::sync::Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    // AtomicBool allows synchronous cancellation checks without await
    is_cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let is_cancelled = Arc::new(AtomicBool::new(false));
        let flag = is_cancelled.clone();
        tokio::spawn(async move {
            match rx.await {
                Ok(_) => flag.store(true, Ordering::Relaxed),
                Err(_) => tracing::debug!("Cancellation receiver dropped before signal"),
            }
        });
        Self {
            cancel_tx: std::sync::Arc::new(std::sync::Mutex::new(Some(tx))),
            is_cancelled,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        if let Ok(mut tx_opt) = self.cancel_tx.lock()
            && let Some(tx) = tx_opt.take()
        {
            match tx.send(()) {
                Ok(_) => tracing::debug!("Cancellation signal sent"),
                Err(_) => tracing::warn!("Failed to send cancellation signal"),
            }
        }
    }
}
