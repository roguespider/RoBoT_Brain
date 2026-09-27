//! Reddit adapter (research_engine.md Tool Catalog: `search_reddit`).
//!
//! Backed by Reddit's public `search.json` endpoint, which needs no API key for
//! JSON access when a descriptive User-Agent is sent. Results are real
//! submissions with real permalinks, titles and selftext, so every citation
//! resolves to the thread it supports (Rule 3 "Source Provenance").
//!
//! Reddit gates automated access by IP reputation and answers a blocked request
//! with an HTML interstitial rather than JSON. That surfaces as a
//! `ProviderUnavailable` naming Reddit, which is the honest outcome: the adapter
//! reports that it could not reach the platform rather than inventing posts.

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://www.reddit.com/search.json";
const PROVIDER: &str = "reddit";

#[derive(Debug, Deserialize)]
struct RedditResponse {
    #[serde(default)]
    data: RedditData,
}

#[derive(Debug, Deserialize, Default)]
struct RedditData {
    #[serde(default)]
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    #[serde(default)]
    data: RedditPost,
}

#[derive(Debug, Deserialize, Default)]
struct RedditPost {
    #[serde(default)]
    title: String,
    #[serde(default)]
    permalink: String,
    #[serde(default)]
    selftext: String,
    #[serde(default)]
    score: i64,
    #[serde(default)]
    num_comments: i64,
    #[serde(default)]
    subreddit: String,
    #[serde(default)]
    is_self: bool,
}

/// Absolute URL for a submission, built from its permalink.
fn permalink_url(permalink: &str) -> Option<String> {
    if permalink.trim().is_empty() {
        return None;
    }
    if permalink.starts_with("http://") || permalink.starts_with("https://") {
        return Some(permalink.to_string());
    }
    Some(format!("https://www.reddit.com{permalink}"))
}

/// Compose a snippet from the fields Reddit actually returns.
///
/// Reddit supplies no abstract, so the honest summary is the submission's own
/// selftext when present, plus its real engagement signals. A post with neither
/// yields a metadata-only snippet rather than an invented description.
fn snippet_for(post: &RedditPost) -> String {
    let body = if post.selftext.trim().is_empty() {
        String::new()
    } else {
        let cleaned = crate::research::sanitize::sanitize_text(&post.selftext);
        crate::research::sanitize::truncate_with_marker(&cleaned, 600).0
    };
    let meta = format!(
        "Posted in r/{} with {} points and {} comments{}.",
        if post.subreddit.is_empty() {
            "unknown"
        } else {
            &post.subreddit
        },
        post.score,
        post.num_comments,
        if post.is_self {
            ", text post"
        } else {
            ", link post"
        }
    );
    if body.is_empty() {
        meta
    } else {
        format!("{body}\n{meta}")
    }
}

pub struct RedditProvider;

impl RedditProvider {
    pub fn new() -> Self {
        Self
    }

    /// Search Reddit, optionally restricted to a single subreddit.
    pub async fn fetch(
        &self,
        query: &str,
        subreddit: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let scope = subreddit
            .map(|s| s.trim().trim_start_matches("r/"))
            .filter(|s| !s.is_empty());
        let url = match scope {
            Some(sub) => format!(
                "https://www.reddit.com/r/{}/search.json?q={}&restrict_sr=on&sort=relevance&limit={}&t=all",
                urlencoding::encode(sub),
                urlencoding::encode(query),
                max_results.clamp(1, 100)
            ),
            None => format!(
                "{ENDPOINT}?q={}&sort=relevance&limit={}&t=all",
                urlencoding::encode(query),
                max_results.clamp(1, 100)
            ),
        };

        let http = crate::research::http::client(PROVIDER)?;
        let response = http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| crate::research::http::transport_error(PROVIDER, e))?;
        if !response.status().is_success() {
            return Err(crate::research::http::status_error(
                PROVIDER,
                response.status(),
            ));
        }
        let parsed: RedditResponse = response
            .json()
            .await
            .map_err(|e| crate::research::http::transport_error(PROVIDER, e))?;

        let posts = parsed.data.children;
        let total = posts.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = posts
            .into_iter()
            .filter_map(|child| {
                let post = child.data;
                if post.title.trim().is_empty() {
                    return None;
                }
                let url = permalink_url(&post.permalink)?;
                if !crate::research::sanitize::is_citable_url(&url) {
                    return None;
                }
                // Reddit sorts by relevance server-side and exposes no score,
                // so position in the accepted sequence carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &post.title,
                    &url,
                    &snippet_for(&post),
                    relevance,
                    SearchSource::Reddit,
                ))
            })
            .collect();

        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for RedditProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for RedditProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = RedditProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, None, 25).await?;
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
        source == SearchSource::Reddit
    }
}
