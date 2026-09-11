#[cfg(feature = "http")]
use crate::research::config::env_key;
#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchResult, SearchSource};

#[cfg(feature = "http")]
pub struct JinaProvider {
    api_key: Option<String>,
}

#[cfg(feature = "http")]
impl JinaProvider {
    pub fn new() -> Self {
        Self {
            api_key: env_key("JINA_API_KEY"),
        }
    }

    pub async fn extract(&self, url: &str) -> Result<String, ResearchError> {
        if self.api_key.is_none() {
            return Err(ResearchError::ProviderUnavailable {
                provider: "jina".into(),
            });
        }
        let key = self.api_key.as_ref().map(|k| k.as_str()).unwrap_or("");
        if key.is_empty() {
            return Err(ResearchError::ProviderUnavailable {
                provider: "jina".into(),
            });
        }
        let client = reqwest::Client::new();
        let resp = client
            .post("https://r.jina.ai/")
            .header("Authorization", format!("Bearer {}", key))
            .body(url.to_string())
            .send()
            .await
            .map_err(|e| ResearchError::ProviderError {
                provider: "jina".into(),
                message: e.to_string(),
            })?;
        let text = resp
            .text()
            .await
            .map_err(|e| ResearchError::ProviderError {
                provider: "jina".into(),
                message: e.to_string(),
            })?;
        Ok(text)
    }

    pub fn rerank(&self, results: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut sorted = results;
        sorted.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}
