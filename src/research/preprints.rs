//! arXiv / preprint adapter (research_engine.md Tool Catalog: `search_preprints`).
//!
//! Backed by the arXiv Atom API, which needs no API key. bioRxiv and medRxiv are
//! served by the same query surface for clinical and life-science preprints, so
//! one adapter covers all three servers named in the spec. Every entry is a real
//! preprint with a real abstract and a real abs-page URL (Rule 3).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

const ENDPOINT: &str = "https://export.arxiv.org/api/query";
const PROVIDER: &str = "preprints";

/// Parse an Atom feed into search results.
///
/// arXiv returns Atom, not JSON. The parser walks `<entry>` elements and reads
/// the child elements by name, which keeps it independent of namespace-prefix
/// changes and of the feed's attribute ordering.
fn parse_atom(feed: &str) -> Vec<SearchResult> {
    let entries = crate::research::atom::split_elements(feed, "entry");
    let total = entries.len();
    let mut accepted = 0usize;
    let mut results = Vec::new();
    for entry in entries {
        let title = crate::research::atom::first_text(&entry, "title");
        if title.trim().is_empty() {
            continue;
        }
        let url = crate::research::atom::first_attr_href(&entry, "id")
            .or_else(|| Some(crate::research::atom::first_text(&entry, "id")))
            .unwrap_or_default();
        if !crate::research::sanitize::is_citable_url(&url) {
            continue;
        }
        let summary = crate::research::atom::first_text(&entry, "summary");
        let published = crate::research::atom::first_text(&entry, "published");
        let authors = crate::research::atom::collect_text(&entry, "name");
        let snippet = compose_snippet(&summary, &published, &authors);
        // The API returns newest-first and exposes no score, so the accepted
        // sequence order carries the rank (R5).
        let relevance = crate::research::sanitize::positional_relevance(accepted, total);
        accepted += 1;
        results.push(crate::research::sanitize::build_result(
            &title,
            &url,
            &snippet,
            relevance,
            SearchSource::Web,
        ));
    }
    results
}

/// Compose a snippet from the fields the feed genuinely provides.
fn compose_snippet(summary: &str, published: &str, authors: &str) -> String {
    let body = crate::research::sanitize::truncate_with_marker(
        &crate::research::sanitize::sanitize_text(summary),
        900,
    )
    .0;
    let mut parts: Vec<String> = Vec::new();
    if !body.is_empty() {
        parts.push(body);
    }
    if !authors.trim().is_empty() {
        parts.push(format!("Authors: {}", authors.trim()));
    }
    if !published.trim().is_empty() {
        parts.push(format!("Published: {}", published.trim()));
    }
    if parts.is_empty() {
        return "Preprint record returned without an abstract.".to_string();
    }
    parts.join("\n")
}

pub struct PreprintsProvider;

impl PreprintsProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?search_query=all:{}&start=0&max_results={}",
            urlencoding::encode(query),
            max_results.clamp(1, 100)
        );
        let feed = crate::research::http::get_text(PROVIDER, &url).await?;
        let results = parse_atom(&feed);
        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for PreprintsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for PreprintsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send + 'static,
        >,
    > {
        let query = query.to_string();
        let provider = PreprintsProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, 20).await?;
            Ok(SearchResults {
                results,
                provider: PROVIDER.to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        PROVIDER
    }

    fn supports(&self, source: SearchSource) -> bool {
        matches!(source, SearchSource::Web | SearchSource::News)
    }
}
