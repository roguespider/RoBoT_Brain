//! Wayback Machine adapter (research_engine.md Tool Catalog: `resurrect_dead_link`).
//!
//! Backed by the Internet Archive's public availability API, which needs no API
//! key and answers whether a URL has an archived snapshot, when the most recent
//! one was taken, and where it can be read. The result is either a genuine
//! archived URL or `NoResults` — it never invents an archive link (Rule 3).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://archive.org/wayback/available";
const PROVIDER: &str = "wayback";

#[derive(Debug, Deserialize)]
struct AvailabilityResponse {
    #[serde(default)]
    archived_snapshots: Snapshots,
}

#[derive(Debug, Deserialize, Default)]
struct Snapshots {
    #[serde(default)]
    closest: Option<Closest>,
}

#[derive(Debug, Deserialize)]
struct Closest {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    available: Option<bool>,
    #[serde(default)]
    status: Option<String>,
}

/// Convert an archive timestamp (`YYYYMMDDhhmmss`) into a readable form.
///
/// Returns the raw value when it does not parse, since a malformed timestamp
/// from the archive is still more useful shown verbatim than discarded.
fn format_timestamp(raw: &str) -> String {
    if raw.len() != 14 {
        return raw.to_string();
    }
    match chrono::NaiveDateTime::parse_from_str(raw, "%Y%m%d%H%M%S") {
        Ok(parsed) => parsed.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        Err(_) => raw.to_string(),
    }
}

pub struct WaybackProvider;

impl WaybackProvider {
    pub fn new() -> Self {
        Self
    }

    /// Look up the closest archived snapshot of a URL.
    ///
    /// This is the `resurrect_dead_link` path: a live URL with no snapshot
    /// reports `NoResults` rather than pointing at a fabricated archive page.
    pub async fn resurrect(&self, url: &str) -> Result<SearchResult, ResearchError> {
        let target = url.trim();
        if !crate::research::sanitize::is_citable_url(target) {
            return Err(ResearchError::NoResults {
                query: format!("wayback:{target}"),
            });
        }
        let request = format!("{ENDPOINT}?url={}", urlencoding::encode(target));
        let parsed: AvailabilityResponse =
            crate::research::http::get_json(PROVIDER, &request).await?;

        let closest =
            parsed
                .archived_snapshots
                .closest
                .ok_or_else(|| ResearchError::NoResults {
                    query: format!("wayback:{target}"),
                })?;

        // The archive answers with `available: false` for a URL it has no
        // snapshot of, so an unavailable answer must not become a citation.
        if closest.available == Some(false) {
            return Err(ResearchError::NoResults {
                query: format!("wayback:{target}"),
            });
        }

        let archived_url = closest
            .url
            .clone()
            .filter(|u| crate::research::sanitize::is_citable_url(u))
            .ok_or_else(|| ResearchError::NoResults {
                query: format!("wayback:{target}"),
            })?;

        let captured = closest
            .timestamp
            .as_deref()
            .map(format_timestamp)
            .unwrap_or_else(|| "unknown capture time".to_string());
        let status = closest
            .status
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("unknown");
        let snippet = format!(
            "Archived snapshot of {target}, captured {captured} (archive reported status: {status})."
        );

        Ok(crate::research::sanitize::build_result(
            &format!("Archived copy of {target}"),
            &archived_url,
            &snippet,
            1.0,
            SearchSource::Web,
        ))
    }
}

impl Default for WaybackProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for WaybackProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = WaybackProvider::new();
        Box::pin(async move {
            // A link-rescue lookup is a single-URL question, so it yields at
            // most one result rather than a ranked list.
            let result = provider.resurrect(&query).await?;
            Ok(SearchResults {
                results: vec![result],
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
