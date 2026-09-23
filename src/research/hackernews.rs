#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct HackerNewsProvider;

#[cfg(feature = "http")]
impl HackerNewsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for HackerNewsProvider {
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
                    title: format!("Hacker News: {}", query),
                    url: format!("https://hn.algolia.com/?q={}", urlencoding::encode(&query)),
                    snippet: format!("Tech community signal for {}", query),
                    relevance: 0.78,
                    source: SearchSource::HN,
                }],
                provider: "hackernews".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "hackernews"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::HN
    }
}
