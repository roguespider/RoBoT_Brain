#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct DetectTrendsProvider;

#[cfg(feature = "http")]
impl DetectTrendsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for DetectTrendsProvider {
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
                    title: format!("Trends: {}", query),
                    url: format!(
                        "https://trends.google.com/explore?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Trending topics across platforms for {}", query),
                    relevance: 0.72,
                    source: SearchSource::Web,
                }],
                provider: "detect_trends".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "detect_trends"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
