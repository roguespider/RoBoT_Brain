//! Mastodon adapter (research_engine.md Tool Catalog: `search_mastodon`).
//!
//! Backed by a public Mastodon instance's `api/v2/search` endpoint, which needs
//! no API key for public content. The instance is configurable so a deployment
//! can point at a reachable server; the default is a well-known public instance.
//! Every result is a real status with a real permalink (Rule 3).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const PROVIDER: &str = "mastodon";

/// Resolve the instance to query.
///
/// `MASTODON_INSTANCE` overrides the default. The value is normalized so callers
/// may supply it with or without a scheme and a trailing slash.
fn instance_base() -> String {
    let configured = crate::research::config::env_key("MASTODON_INSTANCE")
        .unwrap_or_else(|| "https://mastodon.social".to_string());
    let trimmed = configured.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "https://mastodon.social".to_string()
    } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

#[derive(Debug, Deserialize)]
struct MastodonResponse {
    #[serde(default)]
    accounts: Vec<MastodonAccount>,
    #[serde(default)]
    statuses: Vec<MastodonStatus>,
}

#[derive(Debug, Deserialize)]
struct MastodonAccount {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    acct: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    followers_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MastodonStatus {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    favourites_count: Option<u32>,
    #[serde(default)]
    reblogs_count: Option<u32>,
    #[serde(default)]
    account: Option<MastodonStatusAccount>,
}

#[derive(Debug, Deserialize)]
struct MastodonStatusAccount {
    #[serde(default)]
    acct: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
}

pub struct MastodonProvider;

impl MastodonProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let base = instance_base();
        let url = format!(
            "{base}/api/v2/search?q={}&type=statuses&limit={}&resolve=true",
            urlencoding::encode(query),
            max_results.clamp(1, 40)
        );
        let parsed: MastodonResponse = crate::research::http::get_json(PROVIDER, &url).await?;

        // The endpoint returns accounts and statuses; the status list is the
        // evidence, and accounts are reported only as supporting context.
        let statuses = parsed.statuses;
        let account_notes = parsed.accounts.len();
        let total = statuses.len();
        let mut accepted = 0usize;
        let mut results: Vec<SearchResult> = statuses
            .into_iter()
            .filter_map(|status| {
                let status_url = status
                    .url
                    .clone()
                    .filter(|u| crate::research::sanitize::is_citable_url(u))?;
                let body = status.content.clone().unwrap_or_default();
                if body.trim().is_empty() {
                    return None;
                }
                let text = crate::research::sanitize::truncate_with_marker(
                    &crate::research::sanitize::sanitize_text(&body),
                    900,
                )
                .0;
                let author = status.account.unwrap_or(MastodonStatusAccount {
                    acct: None,
                    display_name: None,
                });
                let handle = author.acct.clone().unwrap_or_default();
                let author_label = author
                    .display_name
                    .as_deref()
                    .filter(|d| !d.trim().is_empty())
                    .map(|d| format!("{d} (@{handle})"))
                    .unwrap_or_else(|| format!("@{handle}"));
                let snippet = format!(
                    "{text}\nPosted by {author_label} with {} favourites and {} boosts{}.",
                    status.favourites_count.unwrap_or(0),
                    status.reblogs_count.unwrap_or(0),
                    status
                        .created_at
                        .as_deref()
                        .filter(|c| !c.trim().is_empty())
                        .map(|c| format!(" on {c}"))
                        .unwrap_or_default()
                );
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &format!("Post by {author_label}"),
                    &status_url,
                    &snippet,
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();

        if results.is_empty() && account_notes > 0 {
            // Accounts were matched but no public status was returned. Reporting
            // the accounts is honest supporting context, not a status claim.
            for account in parsed.accounts.into_iter().take(max_results) {
                let Some(url) = account
                    .url
                    .filter(|u| crate::research::sanitize::is_citable_url(u))
                else {
                    continue;
                };
                let note = crate::research::sanitize::truncate_with_marker(
                    &crate::research::sanitize::sanitize_text(&account.note.unwrap_or_default()),
                    600,
                )
                .0;
                let label = account
                    .display_name
                    .as_deref()
                    .filter(|d| !d.trim().is_empty())
                    .map(|d| format!("{d} (@{})", account.acct.clone().unwrap_or_default()))
                    .unwrap_or_else(|| format!("@{}", account.acct.unwrap_or_default()));
                let snippet = format!(
                    "{note}\nMatching account with {} followers; the instance returned no public status for this query.",
                    account.followers_count.unwrap_or(0)
                );
                let relevance = crate::research::sanitize::positional_relevance(
                    accepted,
                    total.max(account_notes),
                );
                accepted += 1;
                results.push(crate::research::sanitize::build_result(
                    &label,
                    &url,
                    &snippet,
                    relevance,
                    SearchSource::Web,
                ));
            }
        }

        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for MastodonProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for MastodonProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = MastodonProvider::new();
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
