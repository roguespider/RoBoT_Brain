use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

pub struct MockProvider {
    results: Vec<SearchResult>,
}

impl MockProvider {
    pub fn new(results: Vec<SearchResult>) -> Self {
        Self { results }
    }
}

impl SearchProvider for MockProvider {
    fn search(
        &self,
        _query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let results = self.results.clone();
        Box::pin(async move {
            Ok(SearchResults {
                results,
                provider: "mock".to_string(),
                query: "mock".to_string(),
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
