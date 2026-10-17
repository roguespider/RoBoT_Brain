//! Google News adapter (research_engine.md Tool Catalog: `search_news`).
//!
//! Backed by the public Google News RSS endpoint, which needs no API key and
//! returns Atom XML containing the real headline, publisher, link and
//! publication date for each story. Every citation resolves to the article it
//! reports (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

const ENDPOINT: &str = "https://news.google.com/rss/search";
const PROVIDER: &str = "news";

/// Parse the RSS document into search results.
fn parse_rss(feed: &str) -> Vec<SearchResult> {
    let items = crate::research::atom::split_elements(feed, "item");
    let total = items.len();
    let mut accepted = 0usize;
    let mut results = Vec::new();
    for item in items {
        let title = crate::research::atom::first_text(&item, "title");
        if title.trim().is_empty() {
            continue;
        }
        let link = crate::research::atom::first_text(&item, "link");
        if !crate::research::sanitize::is_citable_url(&link) {
            continue;
        }
        let publisher = crate::research::atom::first_text(&item, "source");
        let published = crate::research::atom::first_text(&item, "pubDate");
        let description = crate::research::atom::first_text(&item, "description");

        // The description carries Google News' own HTML anchor block; only the
        // nested anchor text is real content, and sanitize_text removes the rest.
        let body = crate::research::sanitize::sanitize_text(&description);
        let snippet = compose_snippet(&body, &publisher, &published);

        // The feed is ordered by recency, and no score is exposed, so the
        // accepted sequence order carries the rank (R5).
        let relevance = crate::research::sanitize::positional_relevance(accepted, total);
        accepted += 1;
        results.push(crate::research::sanitize::build_result(
            &title,
            &link,
            &snippet,
            relevance,
            SearchSource::News,
        ));
    }
    results
}

/// Compose a snippet from the fields the RSS feed genuinely provides.
fn compose_snippet(body: &str, publisher: &str, published: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    let cleaned = crate::research::sanitize::truncate_with_marker(body, 600).0;
    if !cleaned.trim().is_empty() {
        parts.push(cleaned);
    }
    if !publisher.trim().is_empty() {
        parts.push(format!("Publisher: {}", publisher.trim()));
    }
    if !published.trim().is_empty() {
        parts.push(format!("Published: {}", published.trim()));
    }
    if parts.is_empty() {
        return "News article returned without a summary.".to_string();
    }
    parts.join("\n")
}

pub struct NewsProvider;

impl NewsProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?q={}&hl=en-US&gl=US&ceid=US:en",
            urlencoding::encode(query)
        );
        let feed = crate::research::http::get_text(PROVIDER, &url).await?;
        let mut results = parse_rss(&feed);
        // The feed ignores a result count, so the cap is applied locally.
        results.truncate(max_results.clamp(1, 100));
        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for NewsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for NewsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = NewsProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, 25).await?;
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
        source == SearchSource::News
    }
}
