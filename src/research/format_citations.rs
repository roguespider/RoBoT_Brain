#[cfg(feature = "http")]
use crate::research::errors::ResearchError;
#[cfg(feature = "http")]
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

#[cfg(feature = "http")]
pub struct FormatCitationsProvider;

#[cfg(feature = "http")]
impl FormatCitationsProvider {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "http")]
impl SearchProvider for FormatCitationsProvider {
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
                    title: format!("Format citations: {}", query),
                    url: format!(
                        "https://citation.crossref.org/format?doi={}",
                        urlencoding::encode(&query)
                    ),
                    snippet: format!(
                        "Citation formatting (BibTeX/APA/MLA/Chicago/RIS) for {}",
                        query
                    ),
                    relevance: 0.70,
                    source: SearchSource::Web,
                }],
                provider: "format_citations".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }
    fn name(&self) -> &str {
        "format_citations"
    }
    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
