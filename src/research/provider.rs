use crate::research::errors::ResearchError;

/// Search source types for categorizing web search results
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SearchSource {
    /// General web search
    Web,
    /// News search
    News,
    /// Reddit search
    Reddit,
    /// Hacker News search
    HN,
    /// Wikipedia search
    Wikipedia,
}

/// Search query parameters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchQuery {
    /// The search query string
    pub query: String,
    /// The source type to search
    pub source: SearchSource,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Optional language filter
    pub language: Option<String>,
    /// Optional region filter
    pub region: Option<String>,
}

/// Search result item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    /// Title of the result
    pub title: String,
    /// URL of the result
    pub url: String,
    /// Snippet/description of the result
    pub snippet: String,
    /// Relevance score
    pub relevance: f32,
    /// Source type
    pub source: SearchSource,
}

/// Aggregated search results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResults {
    /// List of search results
    pub results: Vec<SearchResult>,
    /// Provider name
    pub provider: String,
    /// Original query string
    pub query: String,
    /// Timestamp of retrieval
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}

/// Search provider trait for implementing search functionality
pub trait SearchProvider: Send + Sync {
    /// Search for the given query and return results
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    >;
    /// Get the name of this provider
    fn name(&self) -> &str;
    /// Check if this provider supports the given search source
    fn supports(&self, source: SearchSource) -> bool;
}
