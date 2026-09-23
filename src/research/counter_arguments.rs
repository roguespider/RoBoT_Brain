#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct CounterArgumentsProvider;

#[cfg(feature = "http")]
impl CounterArgumentsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for CounterArgumentsProvider {
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
                    title: format!("Counter arguments: {}", query),
                    url: format!(
                        "https://scholar.google.com/scholar?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Opposing academic views for {}", query),
                    relevance: 0.72,
                    source: SearchSource::Web,
                }],
                provider: "counter_arguments".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "counter_arguments"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
