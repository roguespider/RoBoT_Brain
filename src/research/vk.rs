//! VK adapter (research_engine.md Tool Catalog: `search_vk`).
//!
//! VK's public API requires an access token, which the platform issues per user
//! and which this tool never holds. Rather than fabricating a post when no token
//! is configured, the adapter reports `ProviderUnavailable` and names the missing
//! credential, so the calling agent knows the tool is unconfigured rather than
//! believing it found a result (Rule 3, Rule 6 "Graceful Degradation").
//!
//! When `VK_ACCESS_TOKEN` is set the adapter performs a real search against the
//! VK news API and returns real post metadata with real permalinks.

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://api.vk.com/method/newsfeed.search";
const PROVIDER: &str = "vk";

#[derive(Debug, Deserialize)]
struct VkResponse {
    #[serde(default)]
    response: Option<VkItems>,
}

#[derive(Debug, Deserialize, Default)]
struct VkItems {
    #[serde(default)]
    items: Vec<VkItem>,
}

#[derive(Debug, Deserialize)]
struct VkItem {
    #[serde(default)]
    id: Option<i64>,
    #[serde(default)]
    owner_id: Option<i64>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    date: Option<i64>,
}

/// Convert a Unix timestamp to an RFC 3339 string, if it is plausible.
fn format_unix_date(seconds: i64) -> Option<String> {
    let naive = chrono::DateTime::from_timestamp(seconds, 0)?;
    Some(naive.to_rfc3339())
}

/// The canonical post URL for a VK item.
fn post_url(owner_id: i64, item_id: i64) -> String {
    format!("https://vk.com/wall{owner_id}_{item_id}")
}

pub struct VkProvider;

impl VkProvider {
    pub fn new() -> Self {
        Self
    }

    /// Search VK's news feed.
    ///
    /// Requires `VK_ACCESS_TOKEN`. Returns `ProviderUnavailable` naming the
    /// token when it is absent, rather than returning an invented post.
    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let Some(token) = crate::research::config::env_key("VK_ACCESS_TOKEN") else {
            return Err(ResearchError::ProviderUnavailable {
                provider: format!("{PROVIDER} (set VK_ACCESS_TOKEN to enable)"),
            });
        };
        let count = max_results.clamp(1, 100);
        let url = format!(
            "{ENDPOINT}?access_token={}&q={}&count={}&v=5.199&extended=1",
            urlencoding::encode(&token),
            urlencoding::encode(query),
            count
        );
        let parsed: VkResponse = crate::research::http::get_json(PROVIDER, &url).await?;
        let items = parsed.response.unwrap_or_default().items;
        let total = items.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = items
            .into_iter()
            .filter_map(|item| {
                let owner_id = item.owner_id?;
                let item_id = item.id?;
                let text = item.text.clone().unwrap_or_default();
                if text.trim().is_empty() {
                    return None;
                }
                let body = crate::research::sanitize::truncate_with_marker(
                    &crate::research::sanitize::sanitize_text(&text),
                    900,
                )
                .0;
                let mut snippet = body;
                if let Some(published) = item.date.and_then(format_unix_date) {
                    snippet = format!("{snippet}\nPublished: {published}");
                }
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &crate::research::sanitize::truncate_with_marker(
                        &crate::research::sanitize::sanitize_text(&text),
                        120,
                    )
                    .0,
                    &post_url(owner_id, item_id),
                    &snippet,
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();
        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for VkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for VkProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = VkProvider::new();
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
        source == SearchSource::Web
    }
}
