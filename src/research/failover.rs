//! Provider failover (research_engine.md R14, Rule 6).
//!
//! Tries each provider in turn and returns the first that yields results. When
//! every provider fails, the returned error names each provider and its cause,
//! so the underlying fault is not lost behind an opaque "all providers failed".

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResults};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Try each provider in order, returning the first non-empty result (R14).
///
/// Failures accumulate and are reported together when the chain is exhausted,
/// which is what Rule 6 "Provider failures return structured errors" requires: a
/// single opaque error would leave the operator unable to tell a rate limit from
/// a DNS failure or a misconfiguration.
pub async fn try_providers(
    providers: &[Arc<dyn SearchProvider>],
    query: &str,
) -> Result<SearchResults, ResearchError> {
    if providers.is_empty() {
        crate::experience::record_research(
            query.to_string(),
            Vec::new(),
            "failover".to_string(),
            Duration::ZERO,
            "failed".to_string(),
        );
        return Err(ResearchError::ProviderUnavailable {
            provider: "no provider is configured for this query".to_string(),
        });
    }

    let started = Instant::now();
    let mut failures: Vec<String> = Vec::new();
    let mut empty_responses: Vec<String> = Vec::new();

    for provider in providers {
        match provider.search(query).await {
            Ok(results) if !results.results.is_empty() => return Ok(results),
            Ok(_) => empty_responses.push(provider.name().to_string()),
            Err(e) => failures.push(format!("{}: {e}", provider.name())),
        }
    }

    // Every provider was reached and none produced evidence (R13.4).
    crate::experience::record_research(
        query.to_string(),
        Vec::new(),
        "failover".to_string(),
        started.elapsed(),
        "failed".to_string(),
    );

    let mut detail: Vec<String> = Vec::new();
    if !failures.is_empty() {
        detail.push(format!("errors: {}", failures.join("; ")));
    }
    if !empty_responses.is_empty() {
        detail.push(format!(
            "returned no results: {}",
            empty_responses.join(", ")
        ));
    }
    Err(ResearchError::ProviderUnavailable {
        provider: if detail.is_empty() {
            "all providers failed".to_string()
        } else {
            format!("all providers failed: {}", detail.join("; "))
        },
    })
}
