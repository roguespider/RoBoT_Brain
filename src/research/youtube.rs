#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct YouTubeProvider;

#[cfg(feature = "http")]
impl YouTubeProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for YouTubeProvider {
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
                    title: format!("YouTube: {}", query),
                    url: format!(
                        "https://youtube.com/results?search_query={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Video content about {}", query),
                    relevance: 0.72,
                    source: SearchSource::Web,
                }],
                provider: "youtube".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "youtube"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
