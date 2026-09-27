//! Shared HTTP transport for research adapters (research_engine.md R9, Rule 5).
//!
//! Centralising the client, timeout and error mapping means every adapter gets
//! the same bounded-timeout behaviour and the same structured error contract, so
//! one adapter cannot hang the pipeline or invent a different failure shape.

use std::sync::OnceLock;
use std::time::Duration;

use crate::research::errors::ResearchError;

/// Shared request timeout for outbound provider calls (R9 Rule 5).
///
/// Bounded so no provider can stall the pipeline; a request that exceeds it is
/// converted into `ResearchError::Timeout` rather than blocking the caller.
pub fn request_timeout() -> Duration {
    static TIMEOUT: OnceLock<Duration> = OnceLock::new();
    *TIMEOUT.get_or_init(|| {
        let secs = crate::research::config::env_key("RESEARCH_HTTP_TIMEOUT_SECS")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(15);
        Duration::from_secs(secs.clamp(1, 120))
    })
}

/// Build a bounded HTTP client with an honest, identifying User-Agent.
///
/// Provider APIs (Reddit, OpenStreetMap Nominatim, SEC EDGAR) reject generic or
/// absent User-Agents, so the agent identifies itself and names the adapter.
pub fn client(provider: &str) -> Result<reqwest::Client, ResearchError> {
    reqwest::Client::builder()
        .timeout(request_timeout())
        .user_agent(format!(
            "robot_brain/0.0.2 (research adapter: {provider}; +https://github.com/roguespider/robot_brain)"
        ))
        .build()
        .map_err(|e| ResearchError::ProviderError {
            provider: provider.to_string(),
            message: format!("failed to build HTTP client: {e}"),
        })
}

/// Map a transport-level reqwest failure into a structured `ResearchError`.
pub fn transport_error(provider: &str, err: reqwest::Error) -> ResearchError {
    if err.is_timeout() {
        ResearchError::Timeout {
            query: provider.to_string(),
            elapsed: request_timeout(),
        }
    } else {
        ResearchError::ProviderError {
            provider: provider.to_string(),
            message: err.to_string(),
        }
    }
}

/// Map a non-success HTTP status into a structured `ResearchError`.
///
/// Auth and rate-limit statuses mean the provider is not usable as configured,
/// which is a distinct condition from a provider-side fault; the two are kept
/// apart so failover can reason about them differently (R14).
pub fn status_error(provider: &str, status: reqwest::StatusCode) -> ResearchError {
    if matches!(
        status,
        reqwest::StatusCode::TOO_MANY_REQUESTS
            | reqwest::StatusCode::FORBIDDEN
            | reqwest::StatusCode::UNAUTHORIZED
    ) {
        ResearchError::ProviderUnavailable {
            provider: provider.to_string(),
        }
    } else {
        ResearchError::ProviderError {
            provider: provider.to_string(),
            message: format!("HTTP status {}", status.as_u16()),
        }
    }
}

/// GET a URL and return the response body as text, enforcing the shared contract.
pub async fn get_text(provider: &str, url: &str) -> Result<String, ResearchError> {
    let http = client(provider)?;
    let response = http
        .get(url)
        .send()
        .await
        .map_err(|e| transport_error(provider, e))?;
    if !response.status().is_success() {
        return Err(status_error(provider, response.status()));
    }
    response
        .text()
        .await
        .map_err(|e| transport_error(provider, e))
}

/// GET a URL and deserialize the body as JSON, enforcing the shared contract.
pub async fn get_json<T: serde::de::DeserializeOwned>(
    provider: &str,
    url: &str,
) -> Result<T, ResearchError> {
    let http = client(provider)?;
    let response = http
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| transport_error(provider, e))?;
    if !response.status().is_success() {
        return Err(status_error(provider, response.status()));
    }
    response
        .json::<T>()
        .await
        .map_err(|e| transport_error(provider, e))
}

/// Require that a provider actually returned evidence.
///
/// An adapter that parses an empty response must report `NoResults`, never an
/// empty success: the calling LLM treats an empty success as a real answer.
pub fn require_results(
    provider: &str,
    query: &str,
    results: Vec<crate::research::provider::SearchResult>,
) -> Result<Vec<crate::research::provider::SearchResult>, ResearchError> {
    if results.is_empty() {
        return Err(ResearchError::NoResults {
            query: format!("{provider}:{query}"),
        });
    }
    Ok(results)
}
