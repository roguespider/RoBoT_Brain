//! Trend detection adapter (research_engine.md Tool Catalog: `detect_trends`).
//!
//! Aggregates real trending topics from the public platforms it is asked about,
//! each through that platform's own public API. A platform that is unreachable
//! or that publishes no matching item is reported as such; trends are never
//! synthesised from the query text (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const PROVIDER: &str = "detect_trends";
const HN_ENDPOINT: &str = "https://hn.algolia.com/api/v1/search?tags=front_page";
const REDDIT_ENDPOINT: &str = "https://www.reddit.com/hot.json?limit=";

#[derive(Debug, Deserialize)]
struct HnFrontPage {
    #[serde(default)]
    hits: Vec<HnTrendHit>,
}

#[derive(Debug, Deserialize)]
struct HnTrendHit {
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

#[derive(Debug, Deserialize)]
struct RedditHot {
    #[serde(default)]
    data: RedditHotData,
}

#[derive(Debug, Deserialize, Default)]
struct RedditHotData {
    #[serde(default)]
    children: Vec<RedditHotChild>,
}

#[derive(Debug, Deserialize)]
struct RedditHotChild {
    #[serde(default)]
    data: RedditHotPost,
}

#[derive(Debug, Deserialize, Default)]
struct RedditHotPost {
    #[serde(default)]
    title: String,
    #[serde(default)]
    permalink: String,
    #[serde(default)]
    score: i64,
    #[serde(default)]
    num_comments: i64,
    #[serde(default)]
    subreddit: String,
}

/// A per-platform outcome, so a partial failure is visible rather than hidden.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformTrendReport {
    /// The platform that was queried.
    pub platform: String,
    /// Whether the platform was reachable and returned items.
    pub available: bool,
    /// Number of trending items retrieved.
    pub item_count: usize,
    /// Why the platform yielded nothing, when it did.
    pub reason: Option<String>,
}

async fn hn_trends() -> Result<HnFrontPage, ResearchError> {
    crate::research::http::get_json(PROVIDER, HN_ENDPOINT).await
}

async fn reddit_trends(limit: usize) -> Result<RedditHot, ResearchError> {
    let url = format!("{REDDIT_ENDPOINT}{}", limit.clamp(1, 100));
    crate::research::http::get_json(PROVIDER, &url).await
}

pub struct DetectTrendsProvider;

impl DetectTrendsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Collect trending items from the named platforms.
    ///
    /// Returns the retrieved items plus a per-platform report, so a caller can
    /// tell the difference between "this platform has no trends" and "this
    /// platform could not be reached".
    pub async fn fetch(
        &self,
        platforms: &[String],
        max_per_platform: usize,
    ) -> (Vec<SearchResult>, Vec<PlatformTrendReport>) {
        let mut all = Vec::new();
        let mut reports = Vec::new();
        let limit = max_per_platform.clamp(1, 50);

        for platform in platforms {
            let name = platform.trim().to_ascii_lowercase();
            if name.is_empty() {
                continue;
            }
            let (results, error) = match name.as_str() {
                "hackernews" | "hn" | "hacker news" => match hn_trends().await {
                    Ok(parsed) => (hn_results(&parsed, &name, limit), None),
                    Err(e) => (Vec::new(), Some(e.to_string())),
                },
                "reddit" => match reddit_trends(limit).await {
                    Ok(parsed) => (reddit_results(&parsed, &name, limit), None),
                    Err(e) => (Vec::new(), Some(e.to_string())),
                },
                other => (
                    Vec::new(),
                    Some(format!(
                        "no trend source is configured for '{other}'; supported platforms are hackernews and reddit"
                    )),
                ),
            };
            let count = results.len();
            reports.push(PlatformTrendReport {
                platform: name,
                available: error.is_none() && count > 0,
                item_count: count,
                reason: error,
            });
            all.extend(results);
        }
        (all, reports)
    }
}

/// Map the HN front page into results, keeping its recency ordering.
fn hn_results(page: &HnFrontPage, platform: &str, limit: usize) -> Vec<SearchResult> {
    let hits = &page.hits;
    let total = hits.len();
    let mut accepted = 0usize;
    let mut results = Vec::new();
    for hit in hits.iter() {
        if accepted >= limit {
            break;
        }
        let Some(title) = hit.title.clone().filter(|t| !t.trim().is_empty()) else {
            continue;
        };
        let url = hit
            .url
            .clone()
            .filter(|u| crate::research::sanitize::is_citable_url(u))
            .or_else(|| {
                hit.object_id
                    .as_deref()
                    .map(|id| format!("https://news.ycombinator.com/item?id={id}"))
            })
            .unwrap_or_default();
        if !crate::research::sanitize::is_citable_url(&url) {
            continue;
        }
        let snippet = format!(
            "Trending on {platform} with {} points and {} comments.",
            hit.points.unwrap_or(0),
            hit.num_comments.unwrap_or(0)
        );
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

/// Map the Reddit hot listing into results, keeping its ranking order.
fn reddit_results(hot: &RedditHot, platform: &str, limit: usize) -> Vec<SearchResult> {
    let posts = &hot.data.children;
    let total = posts.len();
    let mut accepted = 0usize;
    let mut results = Vec::new();
    for child in posts.iter() {
        if accepted >= limit {
            break;
        }
        let post = &child.data;
        if post.title.trim().is_empty() {
            continue;
        }
        let url = if post.permalink.trim().is_empty() {
            String::new()
        } else {
            format!("https://www.reddit.com{}", post.permalink)
        };
        if !crate::research::sanitize::is_citable_url(&url) {
            continue;
        }
        let snippet = format!(
            "Trending on {platform} in r/{} with {} points and {} comments.",
            post.subreddit, post.score, post.num_comments
        );
        let relevance = crate::research::sanitize::positional_relevance(accepted, total);
        accepted += 1;
        results.push(crate::research::sanitize::build_result(
            &post.title,
            &url,
            &snippet,
            relevance,
            SearchSource::Web,
        ));
    }
    results
}

impl Default for DetectTrendsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for DetectTrendsProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            // The adapter takes a list of platforms; a free-text query is
            // reported plainly rather than being turned into a fabricated trend.
            Err(ResearchError::ProviderError {
                provider: PROVIDER.to_string(),
                message: format!(
                    "detect_trends takes a list of platforms, not a free-text query (received: {query})"
                ),
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
