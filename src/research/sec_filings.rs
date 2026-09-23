#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct SecFilingsProvider;

#[cfg(feature = "http")]
impl SecFilingsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for SecFilingsProvider {
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
                    title: format!("SEC EDGAR: {}", query),
                    url: format!(
                        "https://www.sec.gov/edgar/search/?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Financial filings for {}", query),
                    relevance: 0.80,
                    source: SearchSource::Web,
                }],
                provider: "sec_filings".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "sec_filings"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
