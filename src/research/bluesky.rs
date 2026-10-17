//! Bluesky adapter (research_engine.md Tool Catalog: `search_bluesky`).
//!
//! Backed by Bluesky's public `app.bsky.feed.searchPosts` XRPC endpoint, which
//! needs no API key and returns structured JSON for public posts. Every result
//! is a real post with a real permalink, so a citation resolves to the thread it
//! quotes (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://public.api.bsky.app/xrpc/app.bsky.feed.searchPosts";
const PROVIDER: &str = "bluesky";

#[derive(Debug, Deserialize)]
struct BskyResponse {
    #[serde(default)]
    posts: Vec<BskyPost>,
}

#[derive(Debug, Deserialize)]
struct BskyPost {
    #[serde(default)]
    uri: Option<String>,
    #[serde(default)]
    author: Option<BskyAuthor>,
    #[serde(default)]
    record: BskyRecord,
}

#[derive(Debug, Deserialize)]
struct BskyAuthor {
    #[serde(default)]
    handle: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct BskyRecord {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    like_count: Option<u32>,
    #[serde(default)]
    repost_count: Option<u32>,
}

/// Convert an AT-URI to the human-facing bsky.app permalink.
///
/// Posts are addressed as `at://did:plc:xyz/app.bsky.feed.post/abc123`; the web
/// view of that same post is `https://bsky.app/profile/<handle>/post/<rkey>`.
fn post_url(uri: &str, handle: &str) -> Option<String> {
    let rkey = uri.rsplit('/').next().filter(|k| !k.is_empty())?;
    let handle = if handle.trim().is_empty() {
        "self"
    } else {
        handle
    };
    Some(format!(
        "https://bsky.app/profile/{}/post/{}",
        urlencoding::encode(handle),
        urlencoding::encode(rkey)
    ))
}

pub struct BlueskyProvider;

impl BlueskyProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        sort: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        // Bluesky accepts `top` or `latest`; anything else is a caller error and
        // is rejected rather than silently reinterpreted.
        let sort_param = match sort.map(str::trim).filter(|s| !s.is_empty()) {
            None | Some("top") => "top",
            Some("latest") => "latest",
            Some(other) => {
                return Err(ResearchError::ProviderError {
                    provider: PROVIDER.to_string(),
                    message: format!("unsupported sort '{other}'; expected 'top' or 'latest'"),
                });
            }
        };
        let url = format!(
            "{ENDPOINT}?q={}&sort={}&limit={}",
            urlencoding::encode(query),
            sort_param,
            max_results.clamp(1, 100)
        );
        let parsed: BskyResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let posts = parsed.posts;
        let total = posts.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = posts
            .into_iter()
            .filter_map(|post| {
                let uri = post.uri.clone().filter(|u| !u.trim().is_empty())?;
                let author = post.author.unwrap_or(BskyAuthor {
                    handle: None,
                    display_name: None,
                });
                let handle = author.handle.clone().unwrap_or_default();
                let url = post_url(&uri, &handle)?;
                let text = post.record.text.clone().unwrap_or_default();
                if text.trim().is_empty() {
                    return None;
                }
                let body = crate::research::sanitize::truncate_with_marker(
                    &crate::research::sanitize::sanitize_text(&text),
                    900,
                )
                .0;
                let author_label = author
                    .display_name
                    .as_deref()
                    .filter(|d| !d.trim().is_empty())
                    .map(|d| format!("{d} (@{handle})"))
                    .unwrap_or_else(|| format!("@{handle}"));
                let snippet = format!(
                    "{body}\nPosted by {author_label} with {} likes and {} reposts{}.",
                    post.record.like_count.unwrap_or(0),
                    post.record.repost_count.unwrap_or(0),
                    post.record
                        .created_at
                        .as_deref()
                        .filter(|c| !c.trim().is_empty())
                        .map(|c| format!(" on {c}"))
                        .unwrap_or_default()
                );
                // Bluesky returns results in its own relevance order without a
                // score, so the accepted sequence order carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &format!("Post by {author_label}"),
                    &url,
                    &snippet,
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();

        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for BlueskyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for BlueskyProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = BlueskyProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, Some("top"), 25).await?;
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
        source == SearchSource::Web
    }
}
