//! Wikipedia adapter (research_engine.md Tool Catalog: `get_wikipedia`).
//!
//! Backed by the public MediaWiki `list=search` API, which requires no API key
//! and returns structured JSON. Every result is a real article with a real
//! snippet and a real revision URL, so a citation always resolves to the page it
//! claims to support (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://en.wikipedia.org/w/api.php";
const PROVIDER: &str = "wikipedia";

#[derive(Debug, Deserialize)]
struct WikiSearchResponse {
    #[serde(default)]
    query: Option<WikiQuery>,
}

#[derive(Debug, Deserialize)]
struct WikiQuery {
    #[serde(default)]
    search: Vec<WikiHit>,
}

#[derive(Debug, Deserialize)]
struct WikiHit {
    #[serde(default)]
    title: String,
    #[serde(default)]
    snippet: String,
}

/// MediaWiki strips and re-encodes inline HTML inside `snippet`; the response
/// wraps it in `<span class="searchmatch">`. That wrapper is markup, not prose,
/// so it is removed before the text becomes evidence.
fn clean_snippet(raw: &str) -> String {
    let stripped = raw.replace("<span class=\"searchmatch\">", "");
    crate::research::sanitize::sanitize_text(&stripped)
}

/// Build a citable article URL from a Wikipedia page title.
fn article_url(title: &str) -> String {
    format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
}

pub struct WikipediaProvider;

impl WikipediaProvider {
    pub fn new() -> Self {
        Self
    }

    /// Query the MediaWiki search API and map the response into search results.
    ///
    /// Returns `NoResults` rather than an empty success so a caller can never
    /// mistake "Wikipedia has no article" for "Wikipedia returned a finding".
    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?action=query&list=search&srsearch={}&srlimit={}&format=json&utf8=1",
            urlencoding::encode(query),
            max_results.clamp(1, 50)
        );
        let parsed: WikiSearchResponse = crate::research::http::get_json(PROVIDER, &url).await?;

        let hits = parsed
            .query
            .unwrap_or(WikiQuery { search: Vec::new() })
            .search;
        let total = hits.len();
        let results: Vec<SearchResult> = hits
            .into_iter()
            .filter(|hit| !hit.title.trim().is_empty())
            .enumerate()
            .map(|(index, hit)| {
                // MediaWiki returns results best-first but exposes no score, so
                // rank is mapped onto a decaying curve (R5 ranking).
                let relevance = crate::research::sanitize::positional_relevance(index, total);
                crate::research::sanitize::build_result(
                    &hit.title,
                    &article_url(&hit.title),
                    &clean_snippet(&hit.snippet),
                    relevance,
                    SearchSource::Wikipedia,
                )
            })
            .collect();

        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for WikipediaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for WikipediaProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = WikipediaProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, 10).await?;
            Ok(SearchResults {
                results,
                provider: "wikipedia".to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        "wikipedia"
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Wikipedia
    }
}
