//! Brave Search adapter — the secondary provider (research_engine.md R14).
//!
//! Used only as a failover target when the primary DuckDuckGo adapter fails, and
//! only when `BRAVE_API_KEY` is configured. Without a key it reports
//! `ProviderUnavailable` rather than returning an empty success, so the pipeline
//! moves to it only when it can actually answer (R14, Rule 6).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

const ENDPOINT: &str = "https://api.search.brave.com/res/v1/web/search";
const PROVIDER: &str = "brave";

pub struct BraveProvider {
    api_key: Option<String>,
}

impl BraveProvider {
    pub fn new() -> Self {
        Self {
            api_key: crate::research::config::env_key("BRAVE_API_KEY"),
        }
    }
}

impl Default for BraveProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for BraveProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let api_key = self.api_key.clone();
        Box::pin(async move {
            let Some(api_key) = api_key else {
                return Err(ResearchError::ProviderUnavailable {
                    provider: format!("{PROVIDER} (set BRAVE_API_KEY to enable)"),
                });
            };

            let url = format!("{ENDPOINT}?q={}&count=10", urlencoding::encode(&query));
            let http = crate::research::http::client(PROVIDER)?;
            let response = http
                .get(&url)
                .header("Accept", "application/json")
                .header("X-Subscription-Token", &api_key)
                .send()
                .await
                .map_err(|e| crate::research::http::transport_error(PROVIDER, e))?;
            if !response.status().is_success() {
                return Err(crate::research::http::status_error(
                    PROVIDER,
                    response.status(),
                ));
            }
            let payload: serde_json::Value = response
                .json()
                .await
                .map_err(|e| crate::research::http::transport_error(PROVIDER, e))?;

            let items = payload
                .get("web")
                .and_then(|w| w.get("results"))
                .and_then(|r| r.as_array())
                .cloned()
                .unwrap_or_default();
            let total = items.len();
            let mut accepted = 0usize;
            let results: Vec<SearchResult> = items
                .iter()
                .filter_map(|item| {
                    let title = item.get("title")?.as_str()?;
                    let url = item.get("url")?.as_str()?;
                    if !crate::research::sanitize::is_citable_url(url) {
                        return None;
                    }
                    let snippet = item
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");
                    // Brave orders by its own relevance without exposing a
                    // score, so the accepted sequence order carries the rank (R5).
                    let relevance =
                        crate::research::sanitize::positional_relevance(accepted, total);
                    accepted += 1;
                    Some(crate::research::sanitize::build_result(
                        title,
                        url,
                        snippet,
                        relevance,
                        SearchSource::Web,
                    ))
                })
                .collect();

            // An empty Brave response is reported as `NoResults`, so the
            // pipeline can distinguish "no matches" from "provider failed".
            let results = crate::research::http::require_results(PROVIDER, &query, results)?;

            Ok(SearchResults {
                results,
                provider: PROVIDER.to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        PROVIDER
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
