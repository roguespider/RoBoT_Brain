//! Substack adapter (research_engine.md Tool Catalog: `search_substack`).
//!
//! Backed by each publication's public Atom feed, which needs no API key.
//! The input names publications; each is read in turn and its real posts are
//! returned with their real permalinks. A publication with no reachable feed
//! contributes nothing rather than a fabricated entry (Rule 3 "Source Provenance").

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};

const PROVIDER: &str = "substack";

/// Normalize a publication identifier into a feed-capable host.
///
/// Publications are addressed as `name`, `@name`, or a full URL; the host is
/// what the public feed is actually served from.
fn feed_host(publication: &str) -> Option<String> {
    let raw = publication
        .trim()
        .trim_start_matches('@')
        .trim_end_matches('/');
    if raw.is_empty() {
        return None;
    }
    if let Some(rest) = raw
        .strip_prefix("https://")
        .or_else(|| raw.strip_prefix("http://"))
    {
        let host = rest.split('/').next().unwrap_or("");
        if host.is_empty() {
            None
        } else {
            Some(host.to_ascii_lowercase())
        }
    } else if raw.contains('.') && !raw.contains(' ') {
        Some(raw.to_ascii_lowercase())
    } else {
        Some(format!("{raw}.substack.com"))
    }
}

/// Read the posts from one publication's Atom feed.
async fn fetch_publication(
    publication: &str,
    max_posts: usize,
) -> Result<Vec<SearchResult>, ResearchError> {
    let host = feed_host(publication).ok_or_else(|| ResearchError::NoResults {
        query: format!("substack:{publication}"),
    })?;
    let feed_url = format!("https://{host}/feed");
    let body = crate::research::http::get_text(PROVIDER, &feed_url).await?;

    let entries = crate::research::atom::split_elements(&body, "entry");
    let total = entries.len();
    let mut accepted = 0usize;
    let mut results = Vec::new();
    for entry in entries {
        let title = crate::research::atom::first_text(&entry, "title");
        let url = crate::research::atom::first_attr_href(&entry, "link")
            .or_else(|| Some(crate::research::atom::first_text(&entry, "link")))
            .unwrap_or_default();
        if title.trim().is_empty() || !crate::research::sanitize::is_citable_url(&url) {
            continue;
        }
        let summary = crate::research::atom::first_text(&entry, "summary");
        let content = crate::research::atom::first_text(&entry, "content");
        let published = crate::research::atom::first_text(&entry, "published");
        let author = crate::research::atom::first_text(&entry, "name");

        // Feeds carry an abstract or full content; both are real text, and the
        // longer one is the more informative summary.
        let body_text = if content.trim().len() > summary.trim().len() {
            content
        } else {
            summary
        };
        let excerpt = crate::research::sanitize::truncate_with_marker(
            &crate::research::sanitize::sanitize_text(&body_text),
            900,
        )
        .0;

        let mut parts: Vec<String> = Vec::new();
        if !excerpt.trim().is_empty() {
            parts.push(excerpt);
        }
        let mut meta: Vec<String> = Vec::new();
        if !author.trim().is_empty() {
            meta.push(format!("By {author}"));
        }
        if !published.trim().is_empty() {
            meta.push(published.trim().to_string());
        }
        meta.push(format!("Published on {host}"));
        parts.push(meta.join(" - "));

        let relevance = crate::research::sanitize::positional_relevance(accepted, total);
        accepted += 1;
        results.push(crate::research::sanitize::build_result(
            &title,
            &url,
            &parts.join("\n"),
            relevance,
            SearchSource::Web,
        ));
        if results.len() >= max_posts {
            break;
        }
    }
    Ok(results)
}

pub struct SubstackProvider;

impl SubstackProvider {
    pub fn new() -> Self {
        Self
    }

    /// Read posts from one or more comma-separated publications.
    ///
    /// A publication that fails is skipped rather than aborting the batch: one
    /// unreachable blog must not hide the posts of the others. The per-publication
    /// failure is reported as a limitation by the caller through the result set.
    pub async fn fetch(
        &self,
        publications: &str,
        max_posts: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        let requested: Vec<&str> = publications
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if requested.is_empty() {
            return Err(ResearchError::NoResults {
                query: "substack:<no publications specified>".to_string(),
            });
        }
        let per_publication = max_posts.clamp(1, 50) / requested.len().max(1);
        let mut all: Vec<SearchResult> = Vec::new();
        for publication in requested {
            match fetch_publication(publication, per_publication.max(1)).await {
                Ok(mut posts) => all.append(&mut posts),
                Err(e) => {
                    tracing::debug!(
                        publication = %publication,
                        error = %e,
                        "Substack publication could not be read; skipping it"
                    );
                }
            }
        }
        crate::research::http::require_results(PROVIDER, publications, all)
    }
}

impl Default for SubstackProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for SubstackProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = SubstackProvider::new();
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
