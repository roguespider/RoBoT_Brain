#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct NewsProvider;

#[cfg(feature = "http")]
impl NewsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for NewsProvider {
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
                    title: format!("News: {}", query),
                    url: format!(
                        "https://news.google.com/search?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Time-sensitive/current events for {}", query),
                    relevance: 0.80,
                    source: SearchSource::News,
                }],
                provider: "news".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "news"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::News
    }
}
