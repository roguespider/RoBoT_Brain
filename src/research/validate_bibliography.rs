#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct ValidateBibliographyProvider;

#[cfg(feature = "http")]
impl ValidateBibliographyProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for ValidateBibliographyProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            Ok(SearchResults {
                results: vec![SearchResult {
                    title: format!("Validate bibliography: {}", query),
                    url: format!(
                        "https://crossref.org/search?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Reference validation for {}", query),
                    relevance: 0.75,
                    source: SearchSource::Web,
                }],
                provider: "validate_bibliography".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "validate_bibliography"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
