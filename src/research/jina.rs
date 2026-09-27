#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::SearchResult;

#[derive(Debug)]
#[cfg(feature = "http")]
pub struct JinaProvider {
    api_key: Option<String>,
}

#[cfg(feature = "http")]
impl JinaProvider {
    pub fn new() -> Self {
        Self {
            api_key: crate::research::get_config().jina_api_key.clone(),
        }
    }

    /// Whether hosted extraction is configured (R4).
    pub fn is_configured(&self) -> bool {
        crate::research::get_config().has_jina()
    }

    /// Extract clean text for a URL via Jina.
    ///
    /// The extracted text is sanitized before it can enter an evidence packet:
    /// Jina returns the page's readable content, and that content is still
    /// untrusted web data (Rule 9).
    pub async fn extract(&self, url: &str) -> Result<String, ResearchError> {
        let Some(key) = self.api_key.as_deref() else {
            return Err(ResearchError::ProviderUnavailable {
                provider: "jina (set JINA_API_KEY to enable hosted extraction)".into(),
            });
        };
        if !crate::research::sanitize::is_citable_url(url) {
            return Err(ResearchError::ContentExtractionFailed {
                url: url.to_string(),
            });
        }
        let http = crate::research::http::client("jina")?;
        let response = http
            .post("https://r.jina.ai/")
            .header("Authorization", format!("Bearer {key}"))
            .header("Accept", "text/plain")
            .body(url.to_string())
            .send()
            .await
            .map_err(|e| crate::research::http::transport_error("jina", e))?;
        if !response.status().is_success() {
            return Err(crate::research::http::status_error(
                "jina",
                response.status(),
            ));
        }
        let text = response
            .text()
            .await
            .map_err(|e| crate::research::http::transport_error("jina", e))?;
        if text.trim().is_empty() {
            return Err(ResearchError::ContentExtractionFailed {
                url: url.to_string(),
            });
        }
        Ok(crate::research::sanitize::sanitize_text(&text))
    }

    /// Reorder results by relevance (R4 reranking).
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
