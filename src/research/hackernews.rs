//! Hacker News adapter (research_engine.md Tool Catalog: `search_hackernews`).
//!
//! Backed by the public HN Algolia API, which needs no API key and returns
//! structured JSON containing the real story title, URL, author and points.
//! Every result is a genuine submission, so a citation resolves to the thread it
//! claims to support (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://hn.algolia.com/api/v1/search";
const PROVIDER: &str = "hackernews";

#[derive(Debug, Deserialize)]
struct HnResponse {
    #[serde(default)]
    hits: Vec<HnHit>,
}

#[derive(Debug, Deserialize)]
struct HnHit {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    object_id: Option<String>,
    #[serde(default)]
    points: Option<u32>,
    #[serde(default)]
    num_comments: Option<u32>,
}

/// The discussion URL used when a submission is Ask HN or has no outbound link.
fn discussion_url(object_id: &str) -> String {
    format!("https://news.ycombinator.com/item?id={object_id}")
}

/// Compose a snippet from the submission's real engagement signals.
///
/// Algolia returns no excerpt, so the informative fields actually available are
/// the point score and comment count. Reporting those is honest; inventing a
/// summary of the linked page would not be.
fn snippet_for(hit: &HnHit, has_outbound_link: bool) -> String {
    let points = hit.points.unwrap_or(0);
    let comments = hit.num_comments.unwrap_or(0);
    let kind = if has_outbound_link {
        "links to an external article"
    } else {
        "text post (no outbound link)"
    };
    format!("Hacker News submission with {points} points and {comments} comments; {kind}.")
}

pub struct HackerNewsProvider;

impl HackerNewsProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let url = format!(
            "{ENDPOINT}?query={}&tags=(story,comment)&hitsPerPage={}",
            urlencoding::encode(query),
            max_results.clamp(1, 50)
        );
        let parsed: HnResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let total = parsed.hits.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = parsed
            .hits
            .into_iter()
            .filter_map(|hit| {
                // Comments have no title; skipping them keeps every returned row
                // a citable submission rather than an unlabelled comment.
                let title = hit.title.clone().filter(|t| !t.trim().is_empty())?;
                let outbound = hit
                    .url
                    .clone()
                    .filter(|u| crate::research::sanitize::is_citable_url(u));
                let url = outbound
                    .clone()
                    .or_else(|| hit.object_id.as_deref().map(discussion_url))
                    .unwrap_or_default();
                if !crate::research::sanitize::is_citable_url(&url) {
                    return None;
                }
                // Rank by the provider's own ordering: Algolia returns hits best
                // first, so position in the accepted sequence is the rank.
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &url,
                    &snippet_for(&hit, outbound.is_some()),
                    relevance,
                    SearchSource::HN,
                ))
            })
            .collect();

        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for HackerNewsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for HackerNewsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = HackerNewsProvider::new();
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
        source == SearchSource::HN
    }
}
