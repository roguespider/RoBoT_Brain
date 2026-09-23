#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct ScoreReliabilityProvider;

#[cfg(feature = "http")]
impl ScoreReliabilityProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for ScoreReliabilityProvider {
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
                    title: format!("Reliability score: {}", query),
                    url: format!(
                        "https://mediabiasfactcheck.com/search?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Rule-based source quality scoring for {}", query),
                    relevance: 0.68,
                    source: SearchSource::Web,
                }],
                provider: "score_reliability".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "score_reliability"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
