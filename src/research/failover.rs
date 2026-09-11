use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResults};
use std::sync::Arc;
use std::time::Duration;

pub async fn try_providers(
    providers: &[Arc<dyn SearchProvider>],
    query: &str,
) -> Result<SearchResults, ResearchError> {
    let mut errors = Vec::new();
    for provider in providers {
        match provider.search(query).await {
            Ok(results) => return Ok(results),
            Err(e) => errors.push(format!("{}: {}", provider.name(), e)),
        }
    }
    Err(ResearchError::ProviderUnavailable {
        provider: "all".into(),
    })
}

// R13.4: Call record_research with outcome="failed" on total failure
pub fn record_failure(query: &str, sources: Vec<String>, mode: String, duration: Duration) {
    crate::experience::record_research(
        query.to_string(),
        sources,
        mode,
        duration,
        "failed".to_string(),
    );
}
