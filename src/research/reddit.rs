#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct RedditProvider;

#[cfg(feature = "http")]
impl RedditProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for RedditProvider {
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
                    title: format!("Reddit discussion: {}", query),
                    url: format!(
                        "https://reddit.com/search?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Community discussion about {}", query),
                    relevance: 0.75,
                    source: SearchSource::Reddit,
                }],
                provider: "reddit".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "reddit"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Reddit
    }
}
