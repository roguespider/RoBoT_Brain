//! YouTube adapter (research_engine.md Tool Catalog: `search_youtube`).
//!
//! The YouTube Data API v3 is the only supported way to search the catalogue, and
//! it requires an API key. Rather than fabricating a result when no key is
//! configured, this adapter reports `ProviderUnavailable` and names the missing
//! credential, so the calling agent knows the tool is unconfigured rather than
//! believing it found a video (Rule 3, Rule 6 "Graceful Degradation").
//!
//! When `YOUTUBE_API_KEY` is set the adapter performs a real search and returns
//! real video metadata, including a real watch URL per result.

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/search";
const DETAILS_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/videos";
const PROVIDER: &str = "youtube";

#[derive(Debug, Deserialize)]
struct YoutubeSearchResponse {
    #[serde(default)]
    items: Vec<YoutubeSearchItem>,
}

#[derive(Debug, Deserialize)]
struct YoutubeSearchItem {
    #[serde(default)]
    id: Option<YoutubeId>,
    #[serde(default)]
    snippet: Option<YoutubeSnippet>,
}

#[derive(Debug, Deserialize)]
struct YoutubeId {
    #[serde(default, rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct YoutubeSnippet {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    channel_title: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YoutubeDetailsResponse {
    #[serde(default)]
    items: Vec<YoutubeDetailsItem>,
}

#[derive(Debug, Deserialize)]
struct YoutubeDetailsItem {
    #[serde(default)]
    snippet: Option<YoutubeDetailsSnippet>,
    #[serde(default)]
    content_details: Option<YoutubeContentDetails>,
}

#[derive(Debug, Deserialize, Default)]
struct YoutubeDetailsSnippet {
    #[serde(default)]
    channel_title: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct YoutubeContentDetails {
    #[serde(default)]
    duration: Option<String>,
}

/// Convert an ISO-8601 duration (`PT4M13S`) into a readable form.
fn format_duration(raw: &str) -> String {
    let trimmed = raw.trim();
    if !trimmed.starts_with('P') {
        return trimmed.to_string();
    }
    let Some(time_part) = trimmed.find('T').map(|i| &trimmed[i + 1..]) else {
        return trimmed.to_string();
    };
    let mut out = String::new();
    let mut number = String::new();
    for ch in time_part.chars() {
        match ch {
            '0'..='9' => number.push(ch),
            'H' => {
                if let Ok(v) = number.parse::<u32>() {
                    out.push_str(&format!("{v}h "));
                }
                number.clear();
            }
            'M' => {
                if let Ok(v) = number.parse::<u32>() {
                    out.push_str(&format!("{v}m "));
                }
                number.clear();
            }
            'S' => {
                if let Ok(v) = number.parse::<u32>() {
                    out.push_str(&format!("{v}s"));
                }
                number.clear();
            }
            _ => number.clear(),
        }
    }
    let trimmed_out = out.trim().to_string();
    if trimmed_out.is_empty() {
        trimmed.to_string()
    } else {
        trimmed_out
    }
}

pub struct YouTubeProvider;

impl YouTubeProvider {
    pub fn new() -> Self {
        Self
    }

    /// Search the YouTube catalogue.
    ///
    /// Requires `YOUTUBE_API_KEY`. Returns `ProviderUnavailable` naming the key
    /// when it is absent, rather than returning an invented result.
    pub async fn fetch(
        &self,
        query: &str,
        include_transcript: bool,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let Some(api_key) = crate::research::config::env_key("YOUTUBE_API_KEY") else {
            return Err(ResearchError::ProviderUnavailable {
                provider: format!("{PROVIDER} (set YOUTUBE_API_KEY to enable)"),
            });
        };
        let limit = max_results.clamp(1, 50);
        let search_url = format!(
            "{ENDPOINT}?part=snippet&type=video&maxResults={limit}&q={}&key={}",
            urlencoding::encode(query),
            urlencoding::encode(&api_key)
        );
        let parsed: YoutubeSearchResponse =
            crate::research::http::get_json(PROVIDER, &search_url).await?;

        let items = parsed.items;
        let total = items.len();
        let mut accepted = 0usize;
        let mut results: Vec<SearchResult> = items
            .into_iter()
            .filter_map(|item| {
                let video_id = item.id.and_then(|i| i.video_id)?;
                let snippet = item.snippet.unwrap_or_default();
                let title = snippet.title.clone().filter(|t| !t.trim().is_empty())?;
                let description = crate::research::sanitize::truncate_with_marker(
                    &crate::research::sanitize::sanitize_text(
                        &snippet.description.unwrap_or_default(),
                    ),
                    800,
                )
                .0;
                let mut parts: Vec<String> = Vec::new();
                if !description.trim().is_empty() {
                    parts.push(description);
                }
                let mut meta: Vec<String> = Vec::new();
                if let Some(channel) = snippet.channel_title.as_deref().filter(|c| !c.is_empty()) {
                    meta.push(format!("Channel: {channel}"));
                }
                if let Some(published) = snippet.published_at.as_deref().filter(|p| !p.is_empty())
                {
                    meta.push(format!("Published: {published}"));
                }
                if !meta.is_empty() {
                    parts.push(meta.join(" - "));
                }
                if include_transcript {
                    // A transcript requires a separate, authorised download; say
                    // so rather than implying one was retrieved.
                    parts.push(
                        "Transcript not retrieved: YouTube serves transcripts only through an authorised download."
                            .to_string(),
                    );
                }
                let relevance =
                    crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &format!("https://www.youtube.com/watch?v={video_id}"),
                    &parts.join("\n"),
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();

        // Enrich with durations when they are available; a failure here leaves
        // the search results intact rather than discarding real evidence.
        if let Some(durations) = self.fetch_durations(&results, &api_key).await {
            for result in results.iter_mut() {
                if let Some(duration) = durations.get(&result.url) {
                    result.snippet = format!("{duration}\n{}", result.snippet);
                }
            }
        }

        crate::research::http::require_results(PROVIDER, query, results)
    }

    /// Fetch video durations keyed by watch URL.
    ///
    /// Returns `None` when the details call fails, so the caller can keep the
    /// search results it already has.
    async fn fetch_durations(
        &self,
        results: &[SearchResult],
        api_key: &str,
    ) -> Option<std::collections::HashMap<String, String>> {
        let ids: Vec<&str> = results
            .iter()
            .filter_map(|r| r.url.rsplit('=').next())
            .filter(|id| !id.is_empty())
            .collect();
        if ids.is_empty() {
            return None;
        }
        let url = format!(
            "{DETAILS_ENDPOINT}?part=contentDetails,snippet&id={}&key={}",
            urlencoding::encode(&ids.join(",")),
            urlencoding::encode(api_key)
        );
        let parsed: YoutubeDetailsResponse =
            crate::research::http::get_json(PROVIDER, &url).await.ok()?;
        let mut map = std::collections::HashMap::new();
        for (index, item) in parsed.items.iter().enumerate() {
            let Some(id) = ids.get(index) else { continue };
            let duration = item
                .content_details
                .as_ref()
                .and_then(|c| c.duration.as_deref())
                .map(format_duration);
            let channel = item
                .snippet
                .as_ref()
                .and_then(|s| s.channel_title.as_deref())
                .filter(|c| !c.is_empty());
            let label = match (duration, channel) {
                (Some(d), Some(c)) => format!("Duration: {d} - Channel: {c}"),
                (Some(d), None) => format!("Duration: {d}"),
                (None, Some(c)) => format!("Channel: {c}"),
                (None, None) => continue,
            };
            map.insert(format!("https://www.youtube.com/watch?v={id}"), label);
        }
        Some(map)
    }
}

impl Default for YouTubeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for YouTubeProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = YouTubeProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, false, 10).await?;
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
