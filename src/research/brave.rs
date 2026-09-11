#[cfg(feature = "http")]
use crate::research::config::env_key;
#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
#[cfg(feature = "http")]
use reqwest::Client;

#[cfg(feature = "http")]
pub struct BraveProvider {
    api_key: Option<String>,
}

#[cfg(feature = "http")]
impl BraveProvider {
    pub fn new() -> Self {
        Self {
            api_key: env_key("BRAVE_API_KEY"),
        }
    }
}

#[cfg(feature = "http")]
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
            let api_key = match api_key {
                Some(key) => key,
                None => {
                    return Err(ResearchError::ProviderUnavailable {
                        provider: "brave".into(),
                    });
                }
            };

            let client = Client::new();
            let url = format!(
                "https://api.search.brave.com/res/v1/web/search?q={}&count=5",
                urlencoding::encode(&query)
            );

            let resp = client
                .get(&url)
                .header("Accept", "application/json")
                .header("Accept-Encoding", "gzip")
                .header("X-Subscription-Token", &api_key)
                .send()
                .await
                .map_err(|e| ResearchError::ProviderError {
                    provider: "brave".into(),
                    message: e.to_string(),
                })?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(ResearchError::ProviderError {
                    provider: "brave".into(),
                    message: format!("Brave API error {}: {}", status, body),
                });
            }

            let json: serde_json::Value =
                resp.json()
                    .await
                    .map_err(|e| ResearchError::ProviderError {
                        provider: "brave".into(),
                        message: e.to_string(),
                    })?;

            let results = match json.get("web").and_then(|w| w.get("results")) {
                Some(serde_json::Value::Array(items)) => items
                    .iter()
                    .filter_map(|item| {
                        let title = item.get("title")?.as_str()?.to_string();
                        let url = item.get("url")?.as_str()?.to_string();
                        let snippet = item.get("description")?.as_str()?.to_string();
                        Some(SearchResult {
                            title,
                            url,
                            snippet,
                            relevance: 0.8,
                        })
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };

            Ok(SearchResults {
                results,
                provider: "brave".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        "brave"
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
