#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct WikipediaProvider;

#[cfg(feature = "http")]
impl WikipediaProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for WikipediaProvider {
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
                    title: format!("Wikipedia: {}", query),
                    url: format!("https://en.wikipedia.org/wiki/{}", query.replace(" ", "_")),
                    snippet: format!("Encyclopedic information about {}", query),
                    relevance: 0.85,
                    source: SearchSource::Wikipedia,
                }],
                provider: "wikipedia".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "wikipedia"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Wikipedia
    }
}
