#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct DatasetsProvider;

#[cfg(feature = "http")]
impl DatasetsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for DatasetsProvider {
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
                    title: format!("Zenodo/Datasets: {}", query),
                    url: format!(
                        "https://zenodo.org/search?q={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!("Data repository discovery for {}", query),
                    relevance: 0.75,
                    source: SearchSource::Web,
                }],
                provider: "datasets".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "datasets"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
